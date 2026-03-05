use clap::{Parser, Subcommand};
use std::process::ExitCode;
use crate::ctrl::*;

#[derive(Parser)]
#[command(name = "init-ctrl")]
#[command(about = "Control the init process", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all services and their status
    List,
    /// Start a service
    Start { name: String },
    /// Stop a service
    Stop { name: String },
    /// Restart a service
    Restart { name: String },
    /// View service logs
    Logs { name: String },
    /// Reboot the system
    Reboot,
    /// Power off the system
    Poweroff,
}

fn format_duration(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    if days > 0 {
        format!("{}d {:02}:{:02}:{:02}", days, hours, mins, s)
    } else {
        format!("{:02}:{:02}:{:02}", hours, mins, s)
    }
}

pub fn run_ctrl() -> ExitCode {
    let cli = Cli::parse();
    let instruct = match &cli.command {
        Commands::List => Instruct::ListService,
        Commands::Start { name } => Instruct::StartService { name: name.clone() },
        Commands::Stop { name } => Instruct::StopService { name: name.clone() },
        Commands::Restart { name } => Instruct::RestartService { name: name.clone() },
        Commands::Logs { name } => Instruct::GetServiceLog { name: name.clone() },
        Commands::Reboot => Instruct::Reboot,
        Commands::Poweroff => Instruct::PowerOff,
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let target = "127.0.0.1:7788";
        
        match call_init::<serde_json::Value>(target, instruct).await {
            Ok(resp) => {
                if !resp.ok {
                    eprintln!("\x1b[31mError: {}\x1b[0m", resp.data.unwrap_or_default());
                    return ExitCode::FAILURE;
                }

                match cli.command {
                    Commands::List => {
                        let services: Vec<ServiceInfo> = serde_json::from_value(resp.data.unwrap()).unwrap();
                        println!("\x1b[1m{:<20} {:<12} {:<8} {:<20}\x1b[0m", "SERVICE", "STATUS", "PID", "INFO");
                        println!("{}", "-".repeat(65));
                        
                        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();

                        for s in services {
                            let (status_str, pid_str, info) = match s.status {
                                ServiceStatus::Running { pid, start_time } => (
                                    "\x1b[32mRunning\x1b[0m",
                                    pid.to_string(),
                                    format!("Uptime: {}", format_duration(now.saturating_sub(start_time)))
                                ),
                                ServiceStatus::Stopped { exit_code } => (
                                    "\x1b[90mStopped\x1b[0m",
                                    "-".to_string(),
                                    format!("Exit Code: {}", exit_code.map(|c| c.to_string()).unwrap_or_else(|| "N/A".to_string()))
                                ),
                                ServiceStatus::Failed { reason } => (
                                    "\x1b[31mFailed\x1b[0m",
                                    "-".to_string(),
                                    format!("Error: {}", reason)
                                ),
                            };
                            println!("{:<20} {:<12} {:<8} {:<20}", s.name, status_str, pid_str, info);
                        }
                    }
                    Commands::Start { name } => println!("\x1b[32mSuccessfully requested start for service: {}\x1b[0m", name),
                    Commands::Stop { name } => println!("\x1b[33mSuccessfully requested stop for service: {}\x1b[0m", name),
                    Commands::Restart { name } => println!("\x1b[36mSuccessfully requested restart for service: {}\x1b[0m", name),
                    Commands::Logs { name: _ } => {
                        let log: ServiceLog = serde_json::from_value(resp.data.unwrap()).unwrap();
                        println!("{}", log.log);
                    }
                    Commands::Reboot => println!("\x1b[31;1mSystem is rebooting...\x1b[0m"),
                    Commands::Poweroff => println!("\x1b[31;1mSystem is powering off...\x1b[0m"),
                }
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("\x1b[31mError: {}\x1b[0m", err);
                ExitCode::FAILURE
            }
        }
    })
}
