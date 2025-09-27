use rocket::State;
use rocket::serde::json::Json;

use crate::communication::HttpResult;
use crate::setup::SP2AnyVariantInfo;

#[get("/api/meta/sp2any-variant-info")]
pub fn get_api_meta_sp2any_variant(
    variant_info: &State<SP2AnyVariantInfo>,
) -> HttpResult<Json<SP2AnyVariantInfo>> {
    let variant = variant_info.inner().clone();
    Ok(Json(variant))
}
