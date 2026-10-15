use std::io::{Read as _, Write as _};

use super::{Duplex, hex};

#[test]
fn hex_skips_comments_and_reads_pairs() {
    let bytes = hex("stream-rejected");
    assert_eq!(bytes, b"REJECTED EXTERNAL\r\n");
    assert!(hex("no-such-fixture").is_empty());
}

#[test]
fn a_duplex_serves_its_fixture_and_keeps_writes() {
    let mut duplex = Duplex::serving("stream-rejected");
    let mut said = String::new();
    assert!(duplex.read_to_string(&mut said).is_ok());
    assert!(said.starts_with("REJECTED"));
    assert!(
        duplex
            .write_all(b"hi")
            .and_then(|()| duplex.flush())
            .is_ok()
    );
    assert_eq!(duplex.output, b"hi");
}
