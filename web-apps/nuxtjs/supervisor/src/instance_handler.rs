use crate::utils;
use once_cell::sync::Lazy;
use std::sync::RwLock;

#[derive(Debug)]
struct AppState {
    currentMainInstance: String,
    instance1Proc: Option<utils::CommandHandle>,
    instance2Proc: Option<utils::CommandHandle>,
}

static STATE: Lazy<RwLock<AppState>> = Lazy::new(|| {
    RwLock::new(AppState {
        currentMainInstance: "1".to_string(),
        instance1Proc: None,
        instance2Proc: None,
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
            state.instance1Proc = Some(utils::run_cmd_with_logs("/usr/local/share/supervisor/scripts/start_instance.sh", &["1"], &[]));
        }
    }
}
