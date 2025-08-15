use crate::auth;
use crate::auth::Jwt;
use crate::auth::JwtString;
use crate::config::UserConfigDbEntries;
use crate::config::UserConfigForUpdater;
use crate::database;
use crate::model::ApplicationJwtSecret;
use crate::model::ApplicationUserSecrets;
use crate::model::DecryptedDbSecret;
use crate::model::EncryptedDbSecret;
use crate::model::UserLoginCredentials;
use crate::simply_plural;
use crate::updater::UpdaterState;
use crate::webview;
use crate::CliArgs;
use anyhow::{anyhow, Result};
use reqwest::Client;
use rocket::{
    response::{self, content::RawHtml},
    serde::json::Json,
    State,
};
use sqlx::PgPool;

pub async fn run_server(cli_args: CliArgs, client: Client, db_pool: PgPool) -> Result<()> {
    let jwt_secret = ApplicationJwtSecret {
        inner: cli_args.jwt_application_secret.clone(),
    };
    let application_user_secrets = ApplicationUserSecrets {
        inner: cli_args.application_user_secrets.clone(),
    };

    rocket::build()
        .manage(db_pool)
        .manage(cli_args)
        .manage(jwt_secret)
        .manage(application_user_secrets)
        .manage(client)
        .mount(
            "/api",
            routes![
                rest_get_fronting,
                get_updaters_state,
                register,
                login,
                get_config,
                set_config
            ],
        )
        .launch()
        .await
        .map_err(|e| anyhow!(e))
        .map(|_| (()))
}

#[get("/updaters/state")]
const fn get_updaters_state() -> Json<Vec<UpdaterState>> {
    // TODO: Return real updater state
    Json(vec![])
}

#[get("/fronting")]
async fn rest_get_fronting(
    cli_args: &State<CliArgs>,
) -> Result<RawHtml<String>, response::Debug<anyhow::Error>> {
    let config: UserConfigForUpdater = todo!();
    let fronts = simply_plural::fetch_fronts(&config)
        .await
        .map_err(response::Debug)?; // Convert anyhow::Error to response::Debug
    let html = webview::generate_html(&config.system_name, fronts);
    Ok(RawHtml(html))
}

#[post("/user/register", data = "<credentials>")]
async fn register(
    db_pool: &State<PgPool>,
    credentials: Json<UserLoginCredentials>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let pwh = auth::create_password_hash(credentials.password.clone())?;

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

    let jwt_string = auth::verify_password_and_create_token(
        credentials.password.clone(),
        user_info,
        jwt_app_secret,
    )?;

    Ok(Json(jwt_string))
}
// todo. how can we enable users to reset their password? Do I really have to do this all manually here???

#[get("/user/config")]
async fn get_config(
    db_pool: &State<PgPool>,
    jwt: Result<Jwt, response::Debug<anyhow::Error>>,
) -> Result<Json<UserConfigDbEntries<EncryptedDbSecret>>, response::Debug<anyhow::Error>> {
    let user_id = jwt?.user_id()?;

    let user = database::get_user(db_pool, user_id).await?;

    Ok(Json(user.config))
}

#[post("/user/config", data = "<config>")]
async fn set_config(
    config: Json<UserConfigDbEntries<DecryptedDbSecret>>,
    jwt: Result<Jwt, response::Debug<anyhow::Error>>,
    db_pool: &State<PgPool>,
    user_secrets_salt: &State<ApplicationUserSecrets>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let user_id = jwt?.user_id()?;

    let () =
        database::set_user_config_secrets(db_pool, user_id, config.into_inner(), user_secrets_salt)
            .await?;

    Ok(())
}
