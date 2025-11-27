use once_cell::sync::Lazy;
use std::sync::RwLock;
use crate::utils;

// #[derive(Debug)]
// struct AppState {
//     currentMainInstance: 

// }

// static STATE: Lazy<RwLock<AppState>> = Lazy::new(|| {
    
// });

pub struct InstanceHandler {

}

impl InstanceHandler {

    pub async fn startup() {
        utils::run_cmd_with_logs("sleep", &["10"], &[]).await;
        utils::run_cmd_with_logs("echo", &["Hello, world!"], &[]).await;
    }

}
