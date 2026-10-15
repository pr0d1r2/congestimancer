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
