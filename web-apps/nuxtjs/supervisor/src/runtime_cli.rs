use crate::instance_handler::{InstanceHandler, InstanceStatus};
use crate::proxy;
use std::io::Write;
use tokio::io::{self, AsyncBufReadExt, BufReader};

/// Starts the interactive runtime CLI on stdin if available.
pub async fn start() {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    println!("[supervisor] Ready. Type 'help' for available commands.");

    prompt();

    loop {
        match reader.next_line().await {
            Ok(Some(raw)) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    prompt();
                    continue;
                }

                handle_command(trimmed).await;
                prompt();
            }
            Ok(None) => {
                println!("[supervisor] Stdin closed. CLI disabled.");
                break;
            }
            Err(err) => {
                eprintln!("[supervisor] Error reading stdin: {err}");
                break;
            }
        }
    }
}

fn prompt() {
    print!("> ");
    let _ = std::io::stdout().flush();
}

async fn handle_command(cmd_line: &str) {
    let mut parts = cmd_line.split_whitespace();
    let Some(cmd) = parts.next() else {
        return;
    };

    let cmd_lower = cmd.to_ascii_lowercase();

    match cmd_lower.as_str() {
        "help" | "?" => print_help(),
        "status" | "info" => print_status(),
        "instances" => print_instances(),
        "backend" => print_backend(),
        "queue" => print_queue(),
        "update" => handle_update().await,
        "stop" | "shutdown" => handle_stop().await,
        other => println!("[supervisor] Unknown command '{other}'. Type 'help' for options."),
    }
}

fn print_help() {
    println!("[supervisor] Commands:");
    println!("  help/?      Show this help text");
    println!("  status/info Show overall runtime status");
    println!("  instances   Show instance-level information");
    println!("  backend     Show active world backend address");
    println!("  queue       Show update queue information");
    println!("  update      Trigger an update sequence");
    println!("  stop        Stop both instances and exit the supervisor");
}

fn print_status() {
    let status: InstanceStatus = InstanceHandler::status_snapshot();
    let main_instance = status
        .current_main_instance
        .clone()
        .unwrap_or_else(|| "(not assigned)".to_string());

    println!("[supervisor] Main instance: {main_instance}");
    println!(
        "[supervisor] Instances running: #1={} #2={}",
        bool_to_icon(status.instance1_running),
        bool_to_icon(status.instance2_running)
    );
    println!(
        "[supervisor] Update in progress: {}",
        bool_to_icon(status.update_in_progress)
    );
    println!(
        "[supervisor] Pending update requests: {}",
        status.queued_update_requests
    );
}

fn print_instances() {
    let status = InstanceHandler::status_snapshot();
    println!(
        "[supervisor] Instance #1: {}",
        if status.instance1_running {
            "running"
        } else {
            "stopped"
        }
    );
    println!(
        "[supervisor] Instance #2: {}",
        if status.instance2_running {
            "running"
        } else {
            "stopped"
        }
    );
}

fn print_backend() {
    match proxy::current_world_backend() {
        Some(addr) => println!("[supervisor] Active world backend: {addr}"),
        None => println!("[supervisor] Unable to read world backend."),
    }
}

fn print_queue() {
    let status = InstanceHandler::status_snapshot();
    println!(
        "[supervisor] Update in progress: {} | Pending requests: {}",
        bool_to_icon(status.update_in_progress),
        status.queued_update_requests
    );
}

async fn handle_stop() {
    println!("[supervisor] Stop requested. Shutting down instances...");
    InstanceHandler::shutdown().await;
    println!("[supervisor] Shutdown complete. Exiting runtime.");
    std::process::exit(0);
}

async fn handle_update() {
    println!("[supervisor] Update requested.");
    InstanceHandler::on_update().await;
    println!("[supervisor] Update added to queue.");
}

fn bool_to_icon(flag: bool) -> &'static str {
    if flag {
        "yes"
    } else {
        "no"
    }
}
