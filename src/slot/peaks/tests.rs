use std::io;

use super::{learned_peak, learned_peaks, record_peak, reserve_for};
use crate::slot::scratch::Scratch;
use crate::units::{Bytes, Name, parse_name};

fn name(s: &str) -> io::Result<Name> {
    parse_name(s).map_err(io::Error::other)
}

#[test]
fn an_unknown_key_has_no_peak() -> io::Result<()> {
    let scratch = Scratch::new("peaks-unknown")?;
    let dir = scratch.lock_dir()?;
    assert_eq!(
        learned_peak(&dir, &name("gate")?).map_err(io::Error::other)?,
        None
    );
    Ok(())
}

#[test]
fn a_first_peak_is_kept_as_observed() -> io::Result<()> {
    let scratch = Scratch::new("peaks-first")?;
    let dir = scratch.lock_dir()?;
    let key = name("gate")?;
    let kept =
        record_peak(&dir, &key, Bytes(4_000)).map_err(io::Error::other)?;
    assert_eq!(kept, Bytes(4_000));
    let read = learned_peak(&dir, &key).map_err(io::Error::other)?;
    assert_eq!(read, Some(Bytes(4_000)));
    Ok(())
}

#[test]
fn a_bigger_peak_replaces_the_old_one() -> io::Result<()> {
    let scratch = Scratch::new("peaks-bigger")?;
    let dir = scratch.lock_dir()?;
    let key = name("gate")?;
    record_peak(&dir, &key, Bytes(4_000)).map_err(io::Error::other)?;
    let kept =
        record_peak(&dir, &key, Bytes(9_000)).map_err(io::Error::other)?;
    assert_eq!(kept, Bytes(9_000));
    Ok(())
}

#[test]
fn a_smaller_peak_only_pulls_the_old_one_down_a_quarter() -> io::Result<()> {
    let scratch = Scratch::new("peaks-smaller")?;
    let dir = scratch.lock_dir()?;
    let key = name("gate")?;
    record_peak(&dir, &key, Bytes(8_000)).map_err(io::Error::other)?;
    let kept =
        record_peak(&dir, &key, Bytes(1_000)).map_err(io::Error::other)?;
    assert_eq!(kept, Bytes(6_000));
    Ok(())
}

#[test]
fn a_zero_peak_is_not_recorded() -> io::Result<()> {
    let scratch = Scratch::new("peaks-zero")?;
    let dir = scratch.lock_dir()?;
    let key = name("gate")?;
    record_peak(&dir, &key, Bytes(0)).map_err(io::Error::other)?;
    assert_eq!(learned_peak(&dir, &key).map_err(io::Error::other)?, None);
    Ok(())
}

#[test]
fn keys_are_kept_apart_and_listed_by_name() -> io::Result<()> {
    let scratch = Scratch::new("peaks-list")?;
    let dir = scratch.lock_dir()?;
    record_peak(&dir, &name("zeta")?, Bytes(2)).map_err(io::Error::other)?;
    record_peak(&dir, &name("alpha")?, Bytes(1)).map_err(io::Error::other)?;
    let all = learned_peaks(&dir).map_err(io::Error::other)?;
    let shown: Vec<_> =
        all.iter().map(|l| (l.key.as_str(), l.peak.0)).collect();
    assert_eq!(shown, [("alpha", 1), ("zeta", 2)]);
    Ok(())
}

#[test]
fn the_reservation_is_the_peak_plus_a_tenth() {
    assert_eq!(reserve_for(Bytes(1_000)), Bytes(1_100));
    assert_eq!(reserve_for(Bytes(0)), Bytes(0));
    assert_eq!(reserve_for(Bytes(u64::MAX)), Bytes(u64::MAX));
}

#[test]
fn a_slot_file_of_a_key_named_like_a_peak_is_no_peak() -> io::Result<()> {
    let scratch = Scratch::new("peaks-names")?;
    let dir = scratch.lock_dir()?;
    std::fs::write(scratch.path().join("peak.0"), "12")?;
    assert!(learned_peaks(&dir).map_err(io::Error::other)?.is_empty());
    Ok(())
}
