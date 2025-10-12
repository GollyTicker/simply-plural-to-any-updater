use std::{collections::HashMap, sync};

use anyhow::Result;
use sqlx::PgPool;

use crate::{database, platforms, plurality, updater, users};

macro_rules! register_metrics {
    ($pm:ident, $($metric:expr),*) => {
        $(
            $pm.registry().register(Box::new($metric.clone())).unwrap();
        )*
    };
}

#[macro_export]
macro_rules! metric {
    ($metric_type:ty, $metric_name:ident, $metric_name_str:expr, $labels:expr) => {
        pub static $metric_name: std::sync::LazyLock<$metric_type> = {
            use rocket_prometheus::prometheus::opts;
            std::sync::LazyLock::new(|| {
                <$metric_type>::new(opts!($metric_name_str, "."), $labels).unwrap()
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
            stringify!($metric_name).to_lowercase(),
            &["user_id"]
        );
    };
}

#[macro_export]
macro_rules! int_counter_metric {
    ($metric_name:ident) => {
        $crate::metric!(
            rocket_prometheus::prometheus::IntCounterVec,
            $metric_name,
            stringify!($metric_name).to_lowercase(),
            &["user_id"]
        );
    };
}

metric!(
    rocket_prometheus::prometheus::IntGaugeVec,
    SP2ANY_USER_CONFIG_FEATURE,
    "sp2any_user_config_feature",
    &["feature", "status"]
);

pub static PROM_METRICS: sync::LazyLock<rocket_prometheus::PrometheusMetrics> =
    sync::LazyLock::new(|| {
        let promtheus_metrics = rocket_prometheus::PrometheusMetrics::new();

        register_metrics!(
            promtheus_metrics,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_TOTAL_COUNTER,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_FRONTERS_COUNT,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_MEMBERS_COUNT,
            plurality::SIMPLY_PLURAL_FETCH_FRONTS_CUSTOM_FRONTS_COUNT,
            plurality::SIMPLY_PLURAL_WEBSOCKET_CONNECTION_ATTEMPTS_TOTAL,
            plurality::SIMPLY_PLURAL_WEBSOCKET_CONNECTION_ENDED_ERROR_TOTAL,
            plurality::SIMPLY_PLURAL_WEBSOCKET_MESSAGES_RECEIVED_TOTAL,
            plurality::SIMPLY_PLURAL_WEBSOCKET_SEMANTIC_MESSAGES_RECEIVED_TOTAL,
            plurality::SIMPLY_PLURAL_WEBSOCKET_UNKNOWN_MESSAGES_TOTAL,
            updater::UPDATER_MANAGER_RESTART_TOTAL_COUNT,
            updater::UPDATER_MANAGER_RESTART_SUCCESS_COUNT,
            updater::UPDATER_PLATFORM_STATUS,
            platforms::PLURAKIT_API_REQUESTS_TOTAL,
            platforms::PLURAKIT_API_RATELIMIT_REMAINING,
            SP2ANY_USER_CONFIG_FEATURE
        );

        promtheus_metrics
    });

fn count_config_metrics(
    user_config: &users::UserConfigDbEntries<database::Encrypted>,
    feature_counts: &mut HashMap<(String, String), i64>,
) {
    let features = users::metrics_config_values(user_config);

    for (feature_name, is_enabled) in features {
        let status = if is_enabled { "enabled" } else { "disabled" };
        *feature_counts
            .entry((feature_name, status.to_owned()))
            .or_insert(0) += 1;
    }
}

pub async fn collect_user_metrics(
    db_pool: PgPool,
    _: updater::UpdaterManager,
    _: database::ApplicationUserSecrets,
) -> Result<()> {
    log::info!("# | run_user_metrics_job");

    let user_ids = database::get_all_users(&db_pool).await?;

    let mut feature_counts = HashMap::new();

    for user_id in user_ids {
        let user_config = database::get_user(&db_pool, &user_id).await?;
        count_config_metrics(&user_config, &mut feature_counts);
    }

    for ((feature, status), count) in feature_counts {
        SP2ANY_USER_CONFIG_FEATURE
            .with_label_values(&[&feature, &status])
            .set(count);
    }

    log::info!("# | run_user_metrics_job | done");

    Ok(())
}
