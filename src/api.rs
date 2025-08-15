use crate::auth;
use crate::config;
use crate::config::UserConfigDbEntries;
use crate::config::UserConfigForUpdater;
use crate::database;
use crate::model::ApplicationJwtSecret;
use crate::model::DecryptedDbSecret;
use crate::model::EncryptedDbSecret;
use crate::model::JwtString;
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

    rocket::build()
        .manage(db_pool)
        .manage(cli_args)
        .manage(jwt_secret)
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
    let db_user = database::get_user(db_pool, credentials.email.clone())
        .await
        .map_err(response::Debug)?;

    let jwt_string = auth::verify_password_and_create_token(
        credentials.password.clone(),
        db_user,
        jwt_app_secret,
    )?;

    Ok(Json(jwt_string))
}
// todo. how can we enable users to reset their password? Do I really have to do this all manually here???

#[get("/user/config")]
async fn get_config(
    db_pool: &State<PgPool>,
    token: Result<JwtString>,
) -> Result<Json<config::UserConfigDbEntries<EncryptedDbSecret>>, response::Debug<anyhow::Error>> {
    let claims = token.map_err(response::Debug)?;

    // use db_pool and claims.sub instead and get user specific config
    let config: UserConfigDbEntries<EncryptedDbSecret> = todo!();
    // let config = config_store::read_local_config_file(&CliArgs {
    //     config: String::new(),
    //     database_url: String::new(),
    // })
    // .map_err(response::Debug)?;

    Ok(Json(config))
}

#[post("/user/config", data = "<config>")]
async fn set_config(
    db_pool: &State<PgPool>,
    token: Result<JwtString>,
    config: Json<config::UserConfigDbEntries<DecryptedDbSecret>>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let claims = token.map_err(response::Debug)?;
    // use this: db_pool, claims.sub
    todo!()
}
