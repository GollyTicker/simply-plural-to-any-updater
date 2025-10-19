use chrono::{self, DateTime, Utc};
use std::sync;

/**
 * A trait for clocks so that we can mock it in tests.
 *
 * Use the global *now* function to get the current real clock value in production -
 * or the mocked-test-clock in tests.
 */
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> DateTime<Utc> {
        chrono::Utc::now()
    }
}

static GLOBAL_CLOCK: sync::OnceLock<Box<dyn Clock>> = sync::OnceLock::new();

/**
 * The current time via the global clock. `OVerrideable` for tests. Real clock in production.
 */
pub fn now() -> DateTime<Utc> {
    let clock = GLOBAL_CLOCK.get_or_init(|| Box::new(RealClock));
    clock.now()
}

/**
 * This needs to be done before *now* is ever called on the global clock.
 *
 * # Panics
 * When global clock was already before this call.
 * */
#[allow(clippy::unwrap_used)]
pub fn set_global_clock_for_test<C: Clock + 'static>(test_clock: C) {
    GLOBAL_CLOCK
        .set(Box::new(test_clock))
        .map_err(|_| "GLOBAL_CLOCK already initialised.")
        .unwrap();
}
