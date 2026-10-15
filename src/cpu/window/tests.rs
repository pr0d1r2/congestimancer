use super::{LoadWindow, parse_load_window};
use crate::cpu::{CpuError, Loads};
use crate::units::Decimal;

#[test]
fn the_default_window_is_one_minute() {
    assert_eq!(LoadWindow::default(), LoadWindow::One);
}

#[test]
fn windows_parse_by_minutes() {
    let parsed = ["1m", "5m", "15m"].map(|s| parse_load_window(s).ok());
    assert_eq!(
        parsed,
        [
            Some(LoadWindow::One),
            Some(LoadWindow::Five),
            Some(LoadWindow::Fifteen)
        ]
    );
}

#[test]
fn any_other_window_is_an_error_naming_it() {
    let err = parse_load_window("10m");
    assert_eq!(err, Err(CpuError::Window("10m".to_owned())));
    assert_eq!(
        err.map_err(|e| e.to_string()),
        Err("--load-window takes 1m, 5m or 15m, not `10m`".to_owned())
    );
}

#[test]
fn a_window_picks_its_average() {
    let loads = Loads {
        one: Decimal::from_thousandths(1),
        five: Decimal::from_thousandths(5),
        fifteen: Decimal::from_thousandths(15),
    };
    let picked = [LoadWindow::One, LoadWindow::Five, LoadWindow::Fifteen]
        .map(|w| loads.pick(w).thousandths());
    assert_eq!(picked, [1, 5, 15]);
}
