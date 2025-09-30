use std::sync;

use crate::plurality;

macro_rules! register_metrics {
    ($pm:ident, $($metric:expr),*) => {
        $(
            $pm.registry().register(Box::new($metric.clone())).unwrap();
        )*
    };
}

#[macro_export]
macro_rules! metric {
    ($metric_type:ty, $metric_name:ident, $metric_name_str:expr) => {
        pub static $metric_name: std::sync::LazyLock<$metric_type> = {
            use rocket_prometheus::prometheus::opts;
            std::sync::LazyLock::new(|| {
                <$metric_type>::new(opts!($metric_name_str, "."), &["user_id"]).unwrap()
            })
        };
    };
}

#[macro_export]
macro_rules! int_gauge_metric {
    ($metric_name:ident) => {
        $crate::metric!(
            rocket_prometheus::prometheus::IntGaugeVec,
            $metric_name,
            stringify!($metric_name).to_lowercase()
        );
    };
}

#[macro_export]
macro_rules! int_counter_metric {
    ($metric_name:ident) => {
        $crate::metric!(
            rocket_prometheus::prometheus::IntCounterVec,
            $metric_name,
            stringify!($metric_name).to_lowercase()
        );
    };
}

pub static PROM_METRICS: sync::LazyLock<rocket_prometheus::PrometheusMetrics> =
    sync::LazyLock::new(|| {
        let promtheus_metrics = rocket_prometheus::PrometheusMetrics::new();

        register_metrics!(
            promtheus_metrics,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_TOTAL_COUNTER,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_FRONTERS_COUNT,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_MEMBERS_COUNT,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_CUSTOM_FRONTS_COUNT
        );

        promtheus_metrics
    });
