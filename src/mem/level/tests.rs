use super::{available_level, level_within, pressure_level};
use crate::signal::Level;
use crate::units::Percent;

#[test]
fn the_kernel_levels_decode() {
    assert_eq!(pressure_level(1), Some(Level::Normal));
    assert_eq!(pressure_level(2), Some(Level::Warning));
    assert_eq!(pressure_level(4), Some(Level::Critical));
}

#[test]
fn an_unknown_level_is_none() {
    for raw in [0, 3, 5, -1] {
        assert_eq!(pressure_level(raw), None, "{raw}");
    }
}

#[test]
fn mem_level_normal_passes_only_at_normal() {
    assert!(level_within(Level::Normal, Level::Normal));
    assert!(!level_within(Level::Warning, Level::Normal));
    assert!(!level_within(Level::Critical, Level::Normal));
}

#[test]
fn mem_level_warning_passes_at_normal_or_warning() {
    assert!(level_within(Level::Normal, Level::Warning));
    assert!(level_within(Level::Warning, Level::Warning));
    assert!(!level_within(Level::Critical, Level::Warning));
}

#[test]
fn the_available_level_is_a_percent() {
    assert_eq!(available_level(46), Percent::from_hundredths(4600));
    assert_eq!(available_level(0), Percent::from_hundredths(0));
    assert_eq!(available_level(100), Some(Percent::FULL));
}

#[test]
fn an_available_level_out_of_range_is_none() {
    assert_eq!(available_level(-1), None);
    assert_eq!(available_level(101), None);
}
