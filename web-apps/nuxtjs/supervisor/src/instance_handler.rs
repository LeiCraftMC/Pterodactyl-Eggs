use crate::utils;
use once_cell::sync::Lazy;
use std::sync::RwLock;


struct AppState {
    current_main_instance: String,
    instance1_proc: Option<utils::CommandHandle>,
    instance2_proc: Option<utils::CommandHandle>,
}

static STATE: Lazy<RwLock<AppState>> = Lazy::new(|| {
    RwLock::new(AppState {
        current_main_instance: "".to_string(),
        instance1_proc: None,
        instance2_proc: None,
    })
});

pub struct InstanceHandler {}

impl InstanceHandler {
    pub async fn startup() {

        let pull_latest_git_changes_proc = utils::run_cmd_with_logs("/usr/local/share/supervisor/scripts/pull_latest_git_changes.sh", &[], &[]);
        if let Err(e) = pull_latest_git_changes_proc.wait().await {
            eprintln!("Error pulling latest git changes: {}", e);
        }
  
        let cleanup_instances_proc = utils::run_cmd_with_logs("/usr/local/share/supervisor/scripts/cleanup_instances.sh", &[], &[]);
        if let Err(e) = cleanup_instances_proc.wait().await {
            eprintln!("Error cleaning up instances: {}", e);
        }

        let create_new_build_proc = utils::run_cmd_with_logs("/usr/local/share/supervisor/scripts/create_new_build.sh", &[], &[]);
        if let Err(e) = create_new_build_proc.wait().await {
            eprintln!("Error creating new build: {}", e);
        }

        let move_build_to_instance_proc = utils::run_cmd_with_logs("/usr/local/share/supervisor/scripts/move_build_to_instance.sh", &["1"], &[]);
        if let Err(e) = move_build_to_instance_proc.wait().await {
            eprintln!("Error moving build to instance 1: {}", e);
        }

        {
            let mut state = STATE.write().unwrap();
            state.current_main_instance = "1".to_string();
            state.instance1_proc = Some(utils::run_cmd_with_logs("/home/container/.app/instance/1/server/index.mjs", &[], &[
                ("NITRO_PORT", "19131"),
                ("NITRO_HOST", "127.0.0.1")
            ]));
        }
    }
}
