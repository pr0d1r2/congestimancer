use std::error::Error;

use proptest::prelude::*;

use super::{ProcLimit, parse_proc_limit};
use crate::procs::{Glob, ProcsError};
use crate::units::Count;

#[test]
fn parses_pattern_and_count() {
    let limit = parse_proc_limit("nixbld*=8");
    assert_eq!(
        limit,
        Ok(ProcLimit {
            pattern: Glob::new("nixbld*"),
            max: Count(8),
        })
    );
}

#[test]
fn last_equals_splits() {
    let limit = parse_proc_limit("a=b=0").map(|l| l.pattern);
    assert_eq!(limit, Ok(Glob::new("a=b")));
}

#[test]
fn no_equals_is_malformed() {
    let err = parse_proc_limit("cargo");
    assert_eq!(
        err,
        Err(ProcsError::NoEquals {
            input: "cargo".to_owned()
        })
    );
    let shown = err.map_err(|e| e.to_string());
    assert_eq!(
        shown,
        Err("`cargo`: expected PATTERN=N, like cargo=4".into())
    );
}

#[test]
fn empty_pattern_is_malformed() {
    let err = parse_proc_limit("=4");
    assert_eq!(
        err,
        Err(ProcsError::EmptyPattern {
            input: "=4".to_owned()
        })
    );
    assert!(err.is_err_and(|e| e.source().is_none()));
}

#[test]
fn bad_count_is_malformed() {
    for bad in ["cargo=", "cargo=-1", "cargo=1.5", "cargo=x"] {
        let err = parse_proc_limit(bad);
        assert!(matches!(err, Err(ProcsError::Count(_))), "{bad}: {err:?}");
        assert!(err.is_err_and(|e| e.source().is_some()));
    }
}

#[test]
fn renders_back() {
    let limit: Result<ProcLimit, _> = "cc?=12".parse();
    assert_eq!(limit.map(|l| l.to_string()), Ok("cc?=12".to_owned()));
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn round_trips(pattern in "[a-z*?][a-z0-9*?=]{0,12}", n in 0_u64..1000) {
        let text = format!("{pattern}={n}");
        let limit = parse_proc_limit(&text);
        prop_assert_eq!(limit.map(|l| l.to_string()), Ok(text));
    }
}
