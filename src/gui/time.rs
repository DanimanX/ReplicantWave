//! Monotonic animation clock for the GUI.
//!
//! Using `SystemTime::now().duration_since(UNIX_EPOCH).as_secs_f32()` looks
//! harmless but is a precision trap: the Unix timestamp is ~1.79e9 seconds, and
//! an `f32` can only resolve ~213 seconds at that magnitude. Every animated
//! `sin(t * k)` was therefore evaluated once and then frozen — the wavetable
//! and backdrop animations looked completely static.
//!
//! We instead measure from a process-local `Instant`, so `t` stays small and
//! keeps sub-millisecond resolution for hours of runtime.

use std::sync::OnceLock;
use std::time::Instant;

fn start() -> Instant {
    static START: OnceLock<Instant> = OnceLock::new();
    *START.get_or_init(Instant::now)
}

/// Seconds since the first call, with enough precision to animate at 60 FPS.
#[inline]
pub fn now_secs() -> f32 {
    start().elapsed().as_secs_f64() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_advances_within_a_frame_budget() {
        let a = now_secs();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let b = now_secs();
        let delta = b - a;
        assert!(
            delta >= 0.010 && delta < 1.0,
            "animation clock must resolve ~16 ms frames, got {delta}"
        );
    }

    #[test]
    fn clock_is_small_enough_for_f32_animation() {
        // Below ~6 hours an f32 still resolves microseconds, which is plenty.
        assert!(now_secs() < 21_600.0);
    }
}
