use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

pub async fn run_cmd_with_logs(
    cmd: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> tokio::task::JoinHandle<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .envs(env.iter().copied())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn process");

    let stdout = child.stdout.take().expect("no stdout");
    let stderr = child.stderr.take().expect("no stderr");

    // Prefix for nicer output
    let prefix = format!("[{}]", cmd);

    tokio::spawn(async move {
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

        // Wait for exit
        let _ = child.wait().await;
    })
}
