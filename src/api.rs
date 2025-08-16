use crate::auth;
use crate::auth::Jwt;
use crate::auth::JwtString;
use crate::config;
use crate::config::UserConfigDbEntries;
use crate::database;
use crate::model::ApplicationJwtSecret;
use crate::model::ApplicationUserSecrets;
use crate::model::DecryptedDbSecret;
use crate::model::EncryptedDbSecret;
use crate::model::UserId;
use crate::model::UserLoginCredentials;
use crate::setup;
use crate::simply_plural;
use crate::updater_loop;
use crate::updater_state::SharedUpdaters;
use crate::webview;
use anyhow::{anyhow, Result};
use rocket::{
    response::{self, content::RawHtml},
    serde::json::Json,
    State,
};
use sqlx::PgPool;

pub async fn run_server(application_setup: setup::ApplicationSetup) -> Result<()> {
    let setup::ApplicationSetup {
        db_pool,
        client,
        jwt_secret,
        application_user_secrets,
        shared_updaters: shared_updater_state,
    } = application_setup;

    let _ = rocket::build()
        .manage(db_pool)
        .manage(jwt_secret)
        .manage(application_user_secrets)
        .manage(client)
        .manage(shared_updater_state)
        .mount(
            "/api",
            routes![
                rest_get_fronting,
                get_updaters_state,
                restart_updaters,
                register,
                login,
                get_config,
                set_config
            ],
        )
        .launch()
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

#[get("/updaters/state")]
fn get_updaters_state(
    shared_updaters: &State<SharedUpdaters>,
    jwt: Result<Jwt, response::Debug<anyhow::Error>>,
) -> Result<Json<updater_loop::UserUpdatersStatuses>, response::Debug<anyhow::Error>> {
    let user_id = jwt?.user_id()?;

    let updaters_state: updater_loop::UserUpdatersStatuses =
        shared_updaters.get_updaters_state(&user_id)?;

    Ok(Json(updaters_state))
}

#[post("/updaters/restart")]
async fn restart_updaters(
    jwt: Result<Jwt, response::Debug<anyhow::Error>>,
    db_pool: &State<PgPool>,
    application_user_secrets: &State<ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
    shared_updater_state: &State<SharedUpdaters>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let user_id = jwt?.user_id()?;

    let user_config =
        database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;

    let config = config::create_config_with_strong_constraints(&user_id, client, &user_config)?;

    let () = shared_updater_state.restart_updater(&user_id, config)?;

    Ok(())
}

#[get("/fronting/<user_id>")]
async fn rest_get_fronting(
    user_id: &str, // todo. actually use system name here instead of user-id
    db_pool: &State<PgPool>,
    application_user_secrets: &State<ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
) -> Result<RawHtml<String>, response::Debug<anyhow::Error>> {
    eprintln!("GET /fronting/{user_id}.");

    let user_id: UserId = user_id.try_into()?;

    eprintln!("GET /fronting/{}. Getting user secrets", user_id.inner);

    let user_config =
        database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;

    eprintln!("GET /fronting/{}. Creating config", user_id.inner);

    let config = config::create_config_with_strong_constraints(&user_id, client, &user_config)?;

    eprintln!("GET /fronting/{}. Fetching fronts", user_id.inner);

    let fronts = simply_plural::fetch_fronts(&config)
        .await
        .map_err(response::Debug)?;

    eprintln!("GET /fronting/{}. Rendering HTML", user_id.inner);

    let html = webview::generate_html(&config.system_name, fronts);

    eprintln!("GET /fronting/{}. OK", user_id.inner);
    Ok(RawHtml(html))
}

#[post("/user/register", data = "<credentials>")]
async fn register(
    db_pool: &State<PgPool>,
    credentials: Json<UserLoginCredentials>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let pwh = auth::create_password_hash(&credentials.password)?;

    database::create_user(db_pool, credentials.email.clone(), pwh)
        .await
        .map_err(response::Debug)
}

#[post("/user/login", data = "<credentials>")]
async fn login(
    db_pool: &State<PgPool>,
    jwt_app_secret: &State<ApplicationJwtSecret>,
    credentials: Json<UserLoginCredentials>,
) -> Result<Json<JwtString>, response::Debug<anyhow::Error>> {
    let user_id = database::get_user_id(db_pool, credentials.email.clone()).await?;

    let user_info = database::get_user_info(db_pool, user_id)
        .await
        .map_err(response::Debug)?;

    let jwt_string =
        auth::verify_password_and_create_token(&credentials.password, &user_info, jwt_app_secret)?;

    Ok(Json(jwt_string))
}
// todo. how can we enable users to reset their password? Do I really have to do this all manually here???

#[get("/user/config")]
async fn get_config(
    db_pool: &State<PgPool>,
    jwt: Result<Jwt, response::Debug<anyhow::Error>>,
) -> Result<Json<UserConfigDbEntries<EncryptedDbSecret>>, response::Debug<anyhow::Error>> {
    let user_id = jwt?.user_id()?;

    let user_config = database::get_user(db_pool, &user_id).await?;

    Ok(Json(user_config))
}

#[post("/user/config", data = "<config>")]
async fn set_config(
    config: Json<UserConfigDbEntries<DecryptedDbSecret>>,
    jwt: Result<Jwt, response::Debug<anyhow::Error>>,
    db_pool: &State<PgPool>,
    app_user_secrets: &State<ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let user_id = jwt?.user_id()?;

    // check that config satisfies contraints
    let _ = config::create_config_with_strong_constraints(&user_id, client, &config);

    let () =
        database::set_user_config_secrets(db_pool, user_id, config.into_inner(), app_user_secrets)
            .await?;

    Ok(())
}
