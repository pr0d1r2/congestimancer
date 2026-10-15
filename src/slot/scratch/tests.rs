use std::io;

use super::Scratch;

#[test]
fn a_scratch_dir_is_fresh_and_goes_on_drop() -> io::Result<()> {
    let first = Scratch::new("scratch")?;
    let second = Scratch::new("scratch")?;
    assert_ne!(first.path(), second.path());
    let path = first.path().to_owned();
    assert!(path.is_dir());
    drop(first);
    assert!(!path.exists());
    Ok(())
}
