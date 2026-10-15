use super::{Output, Stream, Streams};

#[test]
fn status_and_json_go_to_stdout() {
    assert_eq!(Output::Status.stream(), Stream::Stdout);
    assert_eq!(Output::Json.stream(), Stream::Stdout);
}

#[test]
fn everything_else_goes_to_stderr() {
    for output in [
        Output::Verbose,
        Output::Heartbeat,
        Output::Warning,
        Output::Error,
    ] {
        assert_eq!(output.stream(), Stream::Stderr);
    }
}

#[test]
fn streams_write_each_kind_where_it_belongs() {
    let mut streams = Streams::new(Vec::new(), Vec::new());
    assert!(streams.to(Output::Json).write_all(b"{}").is_ok());
    assert!(streams.to(Output::Warning).write_all(b"w").is_ok());
    assert!(streams.to(Output::Status).write_all(b"s").is_ok());
    let (stdout, stderr) = streams.into_inner();
    assert_eq!(stdout, b"{}s");
    assert_eq!(stderr, b"w");
}
