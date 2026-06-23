//! Platform-conditional monotonic clock.
//!
//! `Instant::now()` panics under `wasm32-unknown-unknown` because that target
//! has no default time source. The rest of `core_engine` calls `now_ms()` and
//! stays oblivious to which backend is live.
//!
//! - **Native:** `Instant::elapsed()` from a process-lifetime origin captured
//!   on first call. Monotonic, zero-allocation.
//! - **wasm32:** imports `engine_now_ms` from the host. The wrapper crate
//!   (`wasm_wrapper`) supplies a JS-side `performance.now()` binding under
//!   that name. Without the import the wasm module will fail to instantiate,
//!   which is the correct failure mode — we want a load-time error, not a
//!   runtime panic deep inside alpha-beta.

/// Monotonic milliseconds since some fixed origin in the process. Only the
/// *difference* between two calls is meaningful.
#[cfg(not(target_arch = "wasm32"))]
#[inline]
pub fn now_ms() -> u64 {
    use std::time::Instant;
    use std::sync::OnceLock;
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    let origin = ORIGIN.get_or_init(Instant::now);
    origin.elapsed().as_millis() as u64
}

#[cfg(target_arch = "wasm32")]
#[inline]
pub fn now_ms() -> u64 {
    // SAFETY: the host (wasm_wrapper) is required to provide this import.
    // If it doesn't, the module won't instantiate.
    unsafe { engine_now_ms() }
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn engine_now_ms() -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_ms_is_monotonic_nondecreasing() {
        let a = now_ms();
        let b = now_ms();
        assert!(b >= a);
    }

    #[test]
    fn now_ms_advances_after_sleep() {
        let a = now_ms();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let b = now_ms();
        assert!(b > a, "expected b > a, got a={a} b={b}");
    }
}
