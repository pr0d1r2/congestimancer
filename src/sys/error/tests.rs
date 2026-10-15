use std::io;

use super::unsupported;

#[test]
fn unsupported_carries_its_reason() {
    let err = unsupported(format_args!("no {} here", "PSI"));
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);
    assert_eq!(err.to_string(), "no PSI here");
}
