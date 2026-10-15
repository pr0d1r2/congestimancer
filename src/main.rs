//! `cgmr`: the command-line entry point. Dispatch only (src:V1): every
//! verb lives in `src/cli`.

fn main() -> std::process::ExitCode {
    congestimancer::cgmr()
}
