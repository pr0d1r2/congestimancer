use super::switch_help;
use crate::cli::table::spec;

#[test]
fn switch_help_names_the_environment_variable() {
    let load = spec("load").map(switch_help).unwrap_or_default();
    assert!(load.ends_with("[env: CGMR_LOAD]"), "{load}");
    let when = spec("when").map(switch_help).unwrap_or_default();
    assert!(!when.contains("env:"), "{when}");
}
