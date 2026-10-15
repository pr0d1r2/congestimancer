use super::{Gpu, whole_percent};
use crate::units::{Bytes, Percent};

#[test]
fn whole_percents_stop_at_100() {
    assert_eq!(whole_percent(0), Percent::from_hundredths(0));
    assert_eq!(whole_percent(100), Some(Percent::FULL));
    assert_eq!(whole_percent(101), None);
    assert_eq!(whole_percent(u64::MAX), None);
}

#[test]
fn a_gpu_holds_what_it_reports() {
    let gpu = Gpu::new(whole_percent(5), Some(Bytes(1)));
    assert_eq!(gpu.busy, whole_percent(5));
    assert_eq!(gpu.vram_free, Some(Bytes(1)));
    assert_eq!(Gpu::default(), Gpu::new(None, None));
}
