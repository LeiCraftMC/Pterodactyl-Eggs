use crate::proxy;
use crate::utils;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
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
        }

        let cleanup_instances_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/cleanup_instances.sh",
            &[],
            &[],
        );
        if let Err(e) = cleanup_instances_proc.wait().await {
            eprintln!("Error cleaning up instances: {}", e);
        }

        let create_new_build_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/create_new_build.sh",
            &[],
            &[],
        );
        if let Err(e) = create_new_build_proc.wait().await {
            eprintln!("Error creating new build: {}", e);
        }

        let move_build_to_instance_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/move_build_to_instance.sh",
            &["1"],
            &[],
        );
        if let Err(e) = move_build_to_instance_proc.wait().await {
            eprintln!("Error moving build to instance 1: {}", e);
        }

        {
            let mut state = STATE.write().unwrap();
            state.current_main_instance = "1".to_string();
        }

        Self::start_instance("1").await;
    }

    pub async fn onUpdate() {
        if let Some(rx) = Self::queue_update_request() {
            if rx.await.is_err() {
                eprintln!("Update request was cancelled before execution");
                return;
            }
        }

        Self::perform_update_sequence().await;

        Self::process_next_queued_update();
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
        }

        let create_new_build_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/create_new_build.sh",
            &[],
            &[],
        );
        if let Err(e) = create_new_build_proc.wait().await {
            eprintln!("Error creating new build: {}", e);
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
        }

        Self::start_instance(new_main_instance).await;

        {
            let mut state = STATE.write().unwrap();
            state.current_main_instance = new_main_instance.to_string();
        }

        // wait a bit to ensure the new instance is fully started
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        //update reverse proxy to point to new instance
        proxy::set_world_backend(if new_main_instance == "1" {
            "127.0.0.1:19131"
        } else {
            "127.0.0.1:19132"
        })
        .await;

        // stop the old instance
        Self::terminate_instance(old_main_instance.as_str()).await;

        let cleanup_old_instance_proc = utils::run_cmd_with_logs(
            "/usr/local/share/supervisor/scripts/cleanup_instance.sh",
            &[old_main_instance.as_str()],
            &[],
        );
        if let Err(e) = cleanup_old_instance_proc.wait().await {
            eprintln!("Error cleaning up instance {}: {}", old_main_instance, e);
        }
    }

    async fn start_instance(instance_number: &str) {
        let mut state = STATE.write().unwrap();

        let instance_path = format!(
            "/home/container/.app/instance/{}/server/index.mjs",
            instance_number
        );

        if instance_number == "1" {
            // check if instance1_proc is already running, if so, error out
            if state.instance1_proc.is_some() {
                eprintln!("Instance 1 is already running.");
                return;
            }

            state.instance1_proc = Some(utils::run_cmd_with_logs(
                instance_path.as_str(),
                &[],
                &[("NITRO_PORT", "19131"), ("NITRO_HOST", "127.0.0.1")],
            ));
        } else if instance_number == "2" {
            // check if instance2_proc is already running, if so, error out
            if state.instance2_proc.is_some() {
                eprintln!("Instance 2 is already running.");
                return;
            }

            state.instance2_proc = Some(utils::run_cmd_with_logs(
                instance_path.as_str(),
                &[],
                &[("NITRO_PORT", "19132"), ("NITRO_HOST", "127.0.0.1")],
            ));
        }
    }

    async fn get_current_main_instance() -> String {
        let state = STATE.read().unwrap();
        state.current_main_instance.clone()
    }

    async fn get_current_instance_proc() -> Option<utils::CommandHandle> {
        let state = STATE.read().unwrap();
        if state.current_main_instance == "1" {
            state.instance1_proc.as_ref().cloned()
        } else if state.current_main_instance == "2" {
            state.instance2_proc.as_ref().cloned()
        } else {
            None
        }
    }

    async fn terminate_instance(instance_number: &str) {
        let mut state = STATE.write().unwrap();

        if instance_number == "1" {
            if let Some(proc) = state.instance1_proc.take() {
                proc.kill().await;
            }
        } else if instance_number == "2" {
            if let Some(proc) = state.instance2_proc.take() {
                proc.kill().await;
            }
        }
    }
}
