use std::fs;
use log::{error, info, warn};
use std::process::{ExitCode};
use std::time::Duration;
use clap::Parser;
use crate::rg35xxp::runner::Next;
use crate::launcher::backend::Backlight;

mod emulate;
#[cfg(feature = "rg35xxp")]
mod rg35xxp;
#[cfg(not(feature = "rg35xxp"))]
mod rg35xxp;
mod launcher;

#[cfg(not(feature = "rg35xxp"))]
pub fn main() -> ExitCode {
    error!("不支持运行环境");
    return ExitCode::FAILURE;
}
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}
#[cfg(feature = "rg35xxp")]
pub fn main() -> ExitCode {
    let args = Args::parse();
    env_logger::init();
    let backend = crate::rg35xxp::backlight::RG35xxpBackend::new().unwrap();
    let mut state = crate::launcher::launcher::Launcher::new(backend.clone()).unwrap();
    loop {
        match rg35xxp::runner::run(&mut state) {
            Ok(Next::Hibernate) => {
                _ = backend.backlight_off();
                if rg35xxp::input::wait_power_key_timeout(Duration::from_secs(30)) {
                    if let Err(e) = fs::write("/sys/power/state", "mem") {
                        error!("Failed to write /sys/power/state: {}", e);
                    }
                }
                _ = backend.backlight_on();
            }
            Ok(Next::Cmd(mut cmd)) => {
                match cmd.status() {
                    Ok(ret) => {
                        info!("程序退出: {}", ret);
                    }
                    Err(err) => {
                        warn!("rg35xxp: runner exited with error: {}", err);
                    }
                }
            },
            Err(err) => {
                warn!("rg35xxp: runner exited with error: {}", err);
                return ExitCode::FAILURE;
            }
        }
    }
}
