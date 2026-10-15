use super::{Failure, UNSUPPORTED, USAGE};

#[test]
fn failures_carry_their_exit_code() {
    assert_eq!(Failure::usage("bad").code, USAGE);
    let unsupported = Failure::unsupported("no source");
    assert_eq!(unsupported.code, UNSUPPORTED);
    assert_eq!(unsupported.message, "no source");
}
