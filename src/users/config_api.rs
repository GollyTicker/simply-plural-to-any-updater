use crate::database;
use crate::http::HttpResult;
use crate::users::config;
use crate::users::jwt;
use rocket::{serde::json::Json, State};
use sqlx::PgPool;

#[get("/api/user/config")]
pub async fn get_api_user_config(
    db_pool: &State<PgPool>,
    jwt: HttpResult<jwt::Jwt>,
) -> HttpResult<Json<config::UserConfigDbEntries<database::Encrypted>>> {
    let user_id = jwt?.user_id()?;

    let user_config = database::get_user(db_pool, &user_id).await?;

    Ok(Json(user_config))
}

#[post("/api/user/config", data = "<config>")]
pub async fn post_api_user_config(
    config: Json<config::UserConfigDbEntries<database::Decrypted>>,
    jwt: HttpResult<jwt::Jwt>,
    db_pool: &State<PgPool>,
    app_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
) -> HttpResult<()> {
    let user_id = jwt?.user_id()?;

    // check that config satisfies contraints
    let (_, valid_db_config) =
        config::create_config_with_strong_constraints(&user_id, client, &config)?;

    let () =
        database::set_user_config_secrets(db_pool, &user_id, valid_db_config, app_user_secrets)
            .await?;

    Ok(())
}
