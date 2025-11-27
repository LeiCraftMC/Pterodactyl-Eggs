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
        let sleep_proc = utils::run_cmd_with_logs("sleep", &["5"], &[]);
        sleep_proc.wait().await.unwrap();

        let echo_proc = utils::run_cmd_with_logs("echo", &["Hello, world!"], &[]);
        echo_proc.detach();
    }
}
