use pluralsync_base::meta::PluralSyncVariantInfo;
use rocket::serde::json::Json;
use rocket::{State, http};

pub type HttpResult<T> = Result<T, (http::Status, String)>;

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn expose_internal_error(err: anyhow::Error) -> (http::Status, String) {
    (http::Status::InternalServerError, err.to_string())
}

#[get("/api/meta/pluralsync-variant-info")]
pub fn get_api_meta_pluralsync_variant(
    variant_info: &State<PluralSyncVariantInfo>,
) -> HttpResult<Json<PluralSyncVariantInfo>> {
    let variant = variant_info.inner().clone();
    log::info!(
        "# | GET /api/meta/pluralsync-variant-info | {}",
        variant.variant
    );
    Ok(Json(variant))
}
