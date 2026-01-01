use crate::proxy;
use crate::utils;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::io::Error;
use std::sync::RwLock;
use tokio::sync::oneshot;

struct AppState {
    current_main_instance: String,
    instance1_proc: Option<utils::CommandHandle>,
    instance2_proc: Option<utils::CommandHandle>,
    update_in_progress: bool,
    queued_update_waiters: VecDeque<oneshot::Sender<()>>,
}

static STATE: Lazy<RwLock<AppState>> = Lazy::new(|| {
    RwLock::new(AppState {
        current_main_instance: "".to_string(),
        instance1_proc: None,
        instance2_proc: None,
        update_in_progress: false,
        queued_update_waiters: VecDeque::new(),
    })
});

#[derive(Clone, Debug)]
pub struct InstanceStatus {
    pub current_main_instance: Option<String>,
    pub instance1_running: bool,
    pub instance2_running: bool,
    pub update_in_progress: bool,
    pub queued_update_requests: usize,
}

pub struct InstanceHandler {}

impl InstanceHandler {
    pub async fn startup() {
        let pull_latest_git_changes_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/pull_latest_git_changes.sh",
            &[],
            &[],
        );
        if let Err(e) = pull_latest_git_changes_proc.wait().await {
            eprintln!("Error pulling latest git changes: {}", e);
            // continue startup even if git pull fails
        }

        let cleanup_instances_result: Result<(), Error> = Self::cleanup_instances().await;
        if let Err(e) = cleanup_instances_result {
            eprintln!("Error cleaning up instances: {}", e);
            // continue startup even if cleanup fails
        }

        let create_new_build_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/create_new_build.sh",
            &[],
            &[],
        );
        if let Err(e) = create_new_build_proc.wait().await {
            eprintln!("Error creating new build: {}", e);
            return;
        }

        let move_build_to_instance_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/move_build_to_instance.sh",
            &["1"],
            &[],
        );
        if let Err(e) = move_build_to_instance_proc.wait().await {
            eprintln!("Error moving build to instance 1: {}", e);
            return;
        }

        {
            let mut state = STATE.write().unwrap();
            state.current_main_instance = "1".to_string();
        }

        Self::start_instance("1").await;
    }

    pub async fn on_update() {
        if let Some(rx) = Self::queue_update_request() {
            if rx.await.is_err() {
                eprintln!("Update request was cancelled before execution");
                return;
            }
        }

        Self::perform_update_sequence().await;

        Self::process_next_queued_update();
    }

    pub fn status_snapshot() -> InstanceStatus {
        let state = STATE.read().unwrap();
        InstanceStatus {
            current_main_instance: if state.current_main_instance.is_empty() {
                None
            } else {
                Some(state.current_main_instance.clone())
            },
            instance1_running: state.instance1_proc.is_some(),
            instance2_running: state.instance2_proc.is_some(),
            update_in_progress: state.update_in_progress,
            queued_update_requests: state.queued_update_waiters.len(),
        }
    }

    pub async fn shutdown() {
        tracing::info!(target: "supervisor", "Shutting down runtime instances");

        Self::terminate_instance("1").await;
        Self::terminate_instance("2").await;

        {
            let mut state = STATE.write().unwrap();
            state.current_main_instance.clear();
            state.update_in_progress = false;
            state.queued_update_waiters.clear();
        }

        let cleanup_instances_result = Self::cleanup_instances().await;
        if let Err(e) = cleanup_instances_result {
            eprintln!("Error cleaning up instances during shutdown: {}", e);
        }
    }

    fn queue_update_request() -> Option<oneshot::Receiver<()>> {
        let mut state = STATE.write().unwrap();
        if state.update_in_progress {
            let (tx, rx) = oneshot::channel();
            state.queued_update_waiters.push_back(tx);
            Some(rx)
        } else {
            state.update_in_progress = true;
            None
        }
    }

    fn process_next_queued_update() {
        loop {
            let next_waiter = {
                let mut state = STATE.write().unwrap();
                if let Some(waiter) = state.queued_update_waiters.pop_front() {
                    Some(waiter)
                } else {
                    state.update_in_progress = false;
                    None
                }
            };

            match next_waiter {
                Some(waiter) => {
                    if waiter.send(()).is_ok() {
                        break;
                    }
                }
                None => break,
            }
        }
    }

    async fn perform_update_sequence() {
        let old_main_instance: String;
        {
            let state = STATE.read().unwrap();
            old_main_instance = state.current_main_instance.clone();
        }
        let new_main_instance = if old_main_instance == "1" { "2" } else { "1" };

        let pull_latest_git_changes_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/pull_latest_git_changes.sh",
            &[],
            &[],
        );
        if let Err(e) = pull_latest_git_changes_proc.wait().await {
            eprintln!("Error pulling latest git changes: {}", e);
            return;
        }

        let create_new_build_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/create_new_build.sh",
            &[],
            &[],
        );
        if let Err(e) = create_new_build_proc.wait().await {
            eprintln!("Error creating new build: {}", e);
            return;
        }

        let move_build_to_instance_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/move_build_to_instance.sh",
            &[new_main_instance],
            &[],
        );
        if let Err(e) = move_build_to_instance_proc.wait().await {
            eprintln!(
                "Error moving build to instance {}: {}",
                new_main_instance, e
            );
            Self::cleanup_instance(new_main_instance).await.ok();
        }

        let startup_success = Self::start_instance(new_main_instance).await;
        if !startup_success {
            eprintln!(
                "Error starting instance {}: startup failed",
                new_main_instance
            );
            Self::cleanup_instance(new_main_instance).await.ok();
            return;
        }
        // wait and check health
        let mut healthy = false;
        for _ in 0..10 {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            healthy = Self::check_instance_health(new_main_instance).await;
            if healthy {
                break;
            }
        }
        if !healthy {
            eprintln!(
                "Instance {} failed health checks after startup",
                new_main_instance
            );
            Self::terminate_instance(new_main_instance).await;
            Self::cleanup_instance(new_main_instance).await.ok();
            return;
        }

        {
            let mut state = STATE.write().unwrap();
            state.current_main_instance = new_main_instance.to_string();
        }

        // wait a bit to ensure the new instance is fully started
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        //update reverse proxy to point to new instance
        if let Err(err) = proxy::set_world_backend(if new_main_instance == "1" {
            "127.0.0.1:19131"
        } else {
            "127.0.0.1:19132"
        }) {
            eprintln!(
                "Error updating reverse proxy to instance {}: {}",
                new_main_instance, err
            );
        }

        // stop the old instance
        Self::terminate_instance(old_main_instance.as_str()).await;

        let cleanup_old_instance_result = Self::cleanup_instance(old_main_instance.as_str()).await;
        if let Err(e) = cleanup_old_instance_result {
            eprintln!("Error cleaning up instance {}: {}", old_main_instance, e);
        }
    }

    async fn start_instance(instance_number: &str) -> bool {
        let mut state = STATE.write().unwrap();

        let instance_path = format!(
            "/home/container/.app/instance/{}/server/index.mjs",
            instance_number
        );
        let instance_args = [instance_path.as_str()];

        if instance_number == "1" {
            // check if instance1_proc is already running, if so, error out
            if state.instance1_proc.is_some() {
                eprintln!("Instance 1 is already running.");
                return false;
            }

            state.instance1_proc = Some(utils::run_cmd_with_logs(
                "bun",
                &instance_args,
                &[("NITRO_PORT", "19131"), ("NITRO_HOST", "127.0.0.1")],
            ));
        } else if instance_number == "2" {
            // check if instance2_proc is already running, if so, error out
            if state.instance2_proc.is_some() {
                eprintln!("Instance 2 is already running.");
                return false;
            }

            state.instance2_proc = Some(utils::run_cmd_with_logs(
                "bun",
                &instance_args,
                &[("NITRO_PORT", "19132"), ("NITRO_HOST", "127.0.0.1")],
            ));
        }
        true
    }

    // async fn get_current_main_instance() -> String {
    //     let state = STATE.read().unwrap();
    //     state.current_main_instance.clone()
    // }

    async fn terminate_instance(instance_number: &str) {
        let proc = {
            let mut state = STATE.write().unwrap();
            if instance_number == "1" {
                state.instance1_proc.take()
            } else if instance_number == "2" {
                state.instance2_proc.take()
            } else {
                None
            }
        };

        if let Some(mut proc) = proc {
            let _ = proc.kill().await;
        }
    }

    async fn cleanup_instance(instance_number: &str) -> Result<(), Error> {
        let cleanup_instance_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/cleanup_instance.sh",
            &[instance_number],
            &[],
        );

        cleanup_instance_proc.wait().await?;

        Ok(())
    }

    async fn cleanup_instances() -> Result<(), Error> {
        let cleanup_instances_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/cleanup_instances.sh",
            &[],
            &[],
        );

        cleanup_instances_proc.wait().await?;

        Ok(())
    }

    async fn check_instance_health(instance_number: &str) -> bool {
        let port = if instance_number == "1" { 19131 } else { 19132 };

        let url = format!("http://127.0.0.1:{}", port);
        let response = match reqwest::get(&url).await {
            Ok(resp) => resp,
            Err(_) => return false,
        };

        let status = response.status().as_u16();
        let healthy_codes = [200..=299, 300..=399, 400..=405];
        healthy_codes.iter().any(|range| range.contains(&status))
    }

}
