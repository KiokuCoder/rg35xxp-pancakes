mod ctrl;
mod cli;
mod daemon;
mod logo;
mod buffer;

use std::process::ExitCode;
use std::path::Path;
use std::env;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let program_name = Path::new(&args[0])
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    if program_name == "init-ctrl" {
        return cli::run_ctrl();
    }

    daemon::run()
}
