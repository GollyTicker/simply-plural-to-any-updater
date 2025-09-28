use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::communication::HttpResult;

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
    Ok(Json(variant))
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SP2AnyVariantInfo {
    pub variant: String,
    pub description: Option<String>,
    pub show_in_ui: bool,
}
