use crate::http::HttpResult;
use crate::platforms::vrchat_auth_types::{
    TwoFactorCodeRequiredResponse, VRChatCredentialsWithCookie, VRChatCredentialsWithTwoFactorAuth,
};
use crate::platforms::{vrchat_auth, vrchat_auth_types};
use crate::users;
use either::Either;
use rocket::serde::json::Json;

#[post("/api/user/platform/vrchat/auth_2fa/request", data = "<creds>")]
pub async fn post_api_user_platform_vrchat_auth_2fa_request(
    creds: Json<vrchat_auth_types::VRChatCredentials>,
    _jwt: users::Jwt, // request should be authenticated, but we don't need user id
) -> HttpResult<Json<Either<VRChatCredentialsWithCookie, TwoFactorCodeRequiredResponse>>> {
    let creds = creds.into_inner();

    let creds_or_tfa_method = vrchat_auth::authenticate_vrchat_for_new_cookie(creds).await?;

    Ok(Json(creds_or_tfa_method))
}

#[post(
    "/api/user/platform/vrchat/auth_2fa/resolve",
    data = "<creds_with_tfa>"
)]
pub async fn post_api_user_platform_vrchat_auth_2fa_resolve(
    _jwt: users::Jwt, // request should be authenticated, but we don't need user id
    creds_with_tfa: Json<VRChatCredentialsWithTwoFactorAuth>,
) -> HttpResult<Json<VRChatCredentialsWithCookie>> {
    let creds_with_tfa = creds_with_tfa.into_inner();

    let valid_creds =
        vrchat_auth::authenticate_vrchat_for_new_cookie_with_2fa(creds_with_tfa).await?;

    Ok(Json(valid_creds))
}
