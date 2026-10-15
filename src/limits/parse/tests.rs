use super::{
    FileNr, limit_share, parse_file_nr, parse_loadavg_tasks, parse_pid_max,
};
use crate::limits::LimitsError;
use crate::units::Percent;

/// The percent of `hundredths` hundredths.
const fn percent(hundredths: u16) -> Option<Percent> {
    Percent::from_hundredths(hundredths)
}

#[test]
fn file_nr_reads_three_tab_separated_numbers() {
    assert_eq!(
        parse_file_nr("4160\t64\t16384\n"),
        Ok(FileNr {
            allocated: 4160,
            unused: 64,
            max: 16384
        })
    );
}

#[test]
fn file_nr_reads_a_max_of_long_max() {
    let nr = parse_file_nr("3008\t0\t9223372036854775807\n");
    assert_eq!(nr.map(|nr| nr.max), Ok(9_223_372_036_854_775_807));
}

#[test]
fn file_nr_with_a_word_is_malformed() {
    assert_eq!(
        parse_file_nr("lots\n"),
        Err(LimitsError::malformed("file-nr", "lots"))
    );
}

#[test]
fn file_nr_with_two_or_four_numbers_is_malformed() {
    assert!(parse_file_nr("1 2").is_err());
    assert!(parse_file_nr("1 2 3 4").is_err());
}

#[test]
fn open_files_are_allocated_less_unused() {
    let nr = FileNr {
        allocated: 4160,
        unused: 64,
        max: 16384,
    };
    assert_eq!(nr.open(), 4096);
}

#[test]
fn more_unused_than_allocated_is_no_open_file() {
    let nr = FileNr {
        allocated: 1,
        unused: 2,
        max: 3,
    };
    assert_eq!(nr.open(), 0);
}

#[test]
fn a_quarter_of_the_max_is_25_percent() {
    assert_eq!(limit_share(4096, 16384), percent(2500));
}

#[test]
fn the_share_rounds_down_to_hundredths() {
    assert_eq!(limit_share(1, 3), percent(3333));
}

#[test]
fn a_huge_max_does_not_overflow() {
    assert_eq!(limit_share(u64::MAX, u64::MAX), percent(10_000));
    assert_eq!(limit_share(3008, u64::MAX >> 1), percent(0));
}

#[test]
fn past_the_max_is_capped_at_100_percent() {
    assert_eq!(limit_share(5, 4), percent(10_000));
}

#[test]
fn a_max_of_zero_has_no_share() {
    assert_eq!(limit_share(0, 0), None);
    assert_eq!(limit_share(1, 0), None);
}

#[test]
fn pid_max_is_one_number() {
    assert_eq!(parse_pid_max("4194304\n"), Ok(4_194_304));
}

#[test]
fn pid_max_with_a_word_is_malformed() {
    assert_eq!(
        parse_pid_max("many\n"),
        Err(LimitsError::malformed("pid_max", "many"))
    );
}

#[test]
fn loadavg_tasks_is_the_total_after_the_slash() {
    let text = "0.52 0.58 0.59 3/8192 12345\n";
    assert_eq!(parse_loadavg_tasks(text), Ok(8192));
}

#[test]
fn loadavg_without_a_task_field_is_malformed() {
    assert_eq!(
        parse_loadavg_tasks("0.00 0.00 0.00\n"),
        Err(LimitsError::malformed("loadavg", "0.00 0.00 0.00"))
    );
}

#[test]
fn loadavg_with_a_bad_total_is_malformed() {
    assert!(parse_loadavg_tasks("0 0 0 3/x 1").is_err());
    assert!(parse_loadavg_tasks("0 0 0 3 1").is_err());
}
