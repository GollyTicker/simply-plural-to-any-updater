use rocket_prometheus::prometheus::{IntCounterVec, IntGaugeVec, opts};

#[allow(clippy::unwrap_used)]
pub static FETCH_FRONTS_TOTAL_COUNTER: std::sync::LazyLock<IntCounterVec> =
    std::sync::LazyLock::new(|| {
        IntCounterVec::new(
            opts!("simply_plural_fetch_total_counter", "."),
            &["user_id"],
        )
        .unwrap()
    });

#[allow(clippy::unwrap_used)]
pub static FETCH_FRONTS_FRONTERS_COUNT: std::sync::LazyLock<IntGaugeVec> =
    std::sync::LazyLock::new(|| {
        IntGaugeVec::new(
            opts!("simply_plural_fetch_fronters_count", "."),
            &["user_id"],
        )
        .unwrap()
    });

#[allow(clippy::unwrap_used)]
pub static FETCH_FRONTS_MEMBERS_COUNT: std::sync::LazyLock<IntGaugeVec> =
    std::sync::LazyLock::new(|| {
        IntGaugeVec::new(
            opts!("simply_plural_fetch_members_count", "."),
            &["user_id"],
        )
        .unwrap()
    });

#[allow(clippy::unwrap_used)]
pub static FETCH_FRONTS_CUSTOM_FRONTS_COUNT: std::sync::LazyLock<IntGaugeVec> =
    std::sync::LazyLock::new(|| {
        IntGaugeVec::new(
            opts!("simply_plural_fetch_custom_fronts_count", "."),
            &["user_id"],
        )
        .unwrap()
    });
