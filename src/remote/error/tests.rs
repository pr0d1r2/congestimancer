use super::RemoteError;

#[test]
fn a_bad_host_is_usage() {
    let err = RemoteError::Host {
        host: "-oProxyCommand=x".to_owned(),
    };
    assert_eq!(err.exit_code(), 64);
    assert_eq!(
        err.to_string(),
        "--host \"-oProxyCommand=x\": not a host name for ssh"
    );
}

fn reason(text: &str) -> String {
    text.to_owned()
}

#[test]
fn a_host_that_cannot_evaluate_is_69() {
    let errs = [
        RemoteError::Schema { found: None },
        RemoteError::Reply {
            reason: reason("x"),
        },
        RemoteError::Spawn {
            reason: reason("x"),
        },
    ];
    for err in errs {
        assert_eq!(err.exit_code(), 69, "{err}");
    }
}

#[test]
fn each_error_says_what_went_wrong() {
    let schema = RemoteError::Schema { found: None }.to_string();
    let want = "remote cgmr speaks JSON schema none, this cgmr reads schema 1";
    assert_eq!(schema, want);
    let reply = RemoteError::Reply {
        reason: reason("eof"),
    }
    .to_string();
    assert_eq!(reply, "remote cgmr reply unreadable: eof");
    let spawn = RemoteError::Spawn {
        reason: reason("gone"),
    }
    .to_string();
    assert_eq!(spawn, "cannot start ssh: gone");
}

#[test]
fn remote_refusals_keep_their_codes() {
    let missing = RemoteError::Missing {
        host: "box".to_owned(),
    };
    assert_eq!(missing.exit_code(), 69);
    assert_eq!(missing.to_string(), "no cgmr on box's PATH");
    let usage = RemoteError::Usage {
        reason: reason("bad --load"),
    };
    assert_eq!(usage.exit_code(), 64);
    assert_eq!(usage.to_string(), "remote cgmr: bad --load");
}
