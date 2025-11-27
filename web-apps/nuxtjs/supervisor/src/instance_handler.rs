use crate::utils;
use once_cell::sync::Lazy;
use std::sync::RwLock;

// #[derive(Debug)]
// struct AppState {
//     currentMainInstance:

// }

// static STATE: Lazy<RwLock<AppState>> = Lazy::new(|| {

// });

pub struct InstanceHandler {}

impl InstanceHandler {
    pub async fn startup() {

        let pull_latest_git_changes_proc = utils::run_cmd_with_logs("/usr/local/share/supervisor/scripts/pull_latest_git_changes.sh", &[], &[]);
        if let Err(e) = pull_latest_git_changes_proc.wait().await {
            eprintln!("Error pulling latest git changes: {}", e);
        }

        let create_new_build_proc = utils::run_cmd_with_logs("/usr/local/share/supervisor/scripts/create_new_build.sh", &[], &[]);
        if let Err(e) = create_new_build_proc.wait().await {
            eprintln!("Error creating new build: {}", e);
        }
    }
}
