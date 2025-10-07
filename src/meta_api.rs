use rocket::serde::json::Json;
use rocket::{State, http};
use sp2any_base::meta::SP2AnyVariantInfo;

pub type HttpResult<T> = Result<T, (http::Status, String)>;

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn expose_internal_error(err: anyhow::Error) -> (http::Status, String) {
    (http::Status::InternalServerError, err.to_string())
}

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
