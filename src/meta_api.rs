use rocket::State;
use rocket::serde::json::Json;
use rocket_prometheus::prometheus::core::Collector;
use serde::{Deserialize, Serialize};

use crate::{communication::HttpResult, plurality};

pub const CANONICAL_SP2ANY_BASE_URL: &str = "https://public-test.sp2any.ayake.net";

pub const SP2ANY_GITHUB_REPOSITORY_URL: &str =
    "https://github.com/GollyTicker/simply-plural-to-any-updater";

pub const SP2ANY_GITHUB_REPOSITORY_RELEASES_URL: &str =
    "https://github.com/GollyTicker/simply-plural-to-any-updater/releases";

#[get("/api/meta/sp2any-variant-info")]
pub fn get_api_meta_sp2any_variant(
    variant_info: &State<SP2AnyVariantInfo>,
) -> HttpResult<Json<SP2AnyVariantInfo>> {
    let variant = variant_info.inner().clone();
    log::info!(
        "# | GET /api/meta/sp2any-variant-info | {}",
        variant.variant
    );
    Ok(Json(variant))
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SP2AnyVariantInfo {
    pub variant: String,
    pub description: Option<String>,
    pub show_in_ui: bool,
}

pub static PROM_METRICS: std::sync::LazyLock<rocket_prometheus::PrometheusMetrics> =
    std::sync::LazyLock::new(|| {
        let pm = rocket_prometheus::PrometheusMetrics::new();

        register_metric(&pm, plurality::FETCH_FRONTS_TOTAL_COUNTER.clone());
        register_metric(&pm, plurality::FETCH_FRONTS_FRONTERS_COUNT.clone());
        register_metric(&pm, plurality::FETCH_FRONTS_MEMBERS_COUNT.clone());
        register_metric(&pm, plurality::FETCH_FRONTS_CUSTOM_FRONTS_COUNT.clone());

        pm
    });

#[allow(clippy::unwrap_used)]
fn register_metric<T: Collector + 'static>(pm: &rocket_prometheus::PrometheusMetrics, x: T) {
    pm.registry().register(Box::new(x)).unwrap();
}
