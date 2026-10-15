use super::{worst_busy, worst_free};
use crate::gpu::Gpu;
use crate::units::{Bytes, Percent};

/// `n` whole percent.
fn pct(n: u16) -> Option<Percent> {
    n.checked_mul(100).and_then(Percent::from_hundredths)
}

#[test]
fn the_busiest_gpu_and_the_least_free_memory_win() {
    let gpus = [
        Gpu::new(pct(10), Some(Bytes(8))),
        Gpu::new(pct(70), Some(Bytes(2))),
        Gpu::new(pct(30), Some(Bytes(5))),
    ];
    assert_eq!(worst_busy(&gpus), pct(70));
    assert_eq!(worst_free(&gpus), Some(Bytes(2)));
}

#[test]
fn gpus_without_a_reading_are_passed_over() {
    let gpus = [Gpu::new(None, Some(Bytes(4))), Gpu::new(pct(5), None)];
    assert_eq!(worst_busy(&gpus), pct(5));
    assert_eq!(worst_free(&gpus), Some(Bytes(4)));
}

#[test]
fn no_reading_anywhere_is_none() {
    assert_eq!(worst_busy(&[]), None);
    assert_eq!(worst_free(&[Gpu::new(pct(1), None)]), None);
}
