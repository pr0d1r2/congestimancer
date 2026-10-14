//! `congestimancer-dev`: entry point. Dispatch lives in the library.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(congestimancer_dev::run(&args, std::path::Path::new(".")))
}
