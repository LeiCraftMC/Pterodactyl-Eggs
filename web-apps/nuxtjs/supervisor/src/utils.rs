use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};

pub struct CommandHandle {
    child: Child,
    log_task: tokio::task::JoinHandle<()>,
}

impl CommandHandle {
    pub async fn wait(mut self) -> std::io::Result<std::process::ExitStatus> {
        let status = self.child.wait().await;
        let _ = self.log_task.await;
        status
    }

    pub async fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill().await
    }

    pub fn detach(mut self) {
        tokio::spawn(async move {
            let _ = self.child.wait().await;
            let _ = self.log_task.await;
        });
    }
}

pub fn run_cmd_with_logs(cmd: &str, args: &[&str], env: &[(&str, &str)]) -> CommandHandle {
    let mut child = Command::new(cmd)
        .args(args)
        .envs(env.iter().copied())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn process");

    let stdout = child.stdout.take().expect("no stdout");
    let stderr = child.stderr.take().expect("no stderr");

    let prefix = format!("[{}]", cmd);

    let log_task = tokio::spawn(async move {
        let mut out_reader = BufReader::new(stdout).lines();
        let mut err_reader = BufReader::new(stderr).lines();

        loop {
            tokio::select! {
                line = out_reader.next_line() => {
                    if let Ok(Some(l)) = line {
                        println!("{} {}", prefix, l);
                    } else {
                        break;
                    }
                }
                line = err_reader.next_line() => {
                    if let Ok(Some(l)) = line {
                        eprintln!("{} {}", prefix, l);
                    } else {
                        break;
                    }
                }
            }
        }
    });

    CommandHandle { child, log_task }
}
