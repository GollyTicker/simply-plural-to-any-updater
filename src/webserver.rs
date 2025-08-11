use crate::config::Config;
use crate::config_store;
use crate::database;
use crate::simply_plural;
use crate::updater::UpdaterState;
use crate::CliArgs;
use anyhow::{anyhow, Result};
use argon2::PasswordHash;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use rocket::{
    response::{self, content::RawHtml},
    serde::json::Json,
    State,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::jwt;

#[derive(Serialize, Deserialize)]
pub struct UserCredentials {
    username: String,
    password: String,
}

pub async fn run_server(cli_args: CliArgs, config: Config, db_pool: PgPool) -> Result<()> {
    rocket::build()
        .manage(config)
        .manage(cli_args)
        .manage(db_pool)
        .mount("/", routes![rest_get_fronting])
        .mount("/api", routes![get_updaters_state])
        .mount("/api/auth", routes![register, login])
        .mount("/api/config", routes![get_config, set_config])
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
    config: &State<Config>,
) -> Result<RawHtml<String>, response::Debug<anyhow::Error>> {
    let fronts = simply_plural::fetch_fronts(config.inner())
        .await
        .map_err(response::Debug)?; // Convert anyhow::Error to response::Debug
    let html = generate_html(config.inner(), fronts);
    Ok(RawHtml(html))
}

fn generate_html(config: &Config, fronts: Vec<simply_plural::Fronter>) -> String {
    let fronts_formatted = fronts
        .into_iter()
        .map(|m| -> String {
            format!(
                "<div><img src=\"{}\" /><p>{}</p></div>",
                m.avatar_url, // if URL is empty, then simply no image is rendered.
                html_escape::encode_text(&m.name)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        r"<html>
    <head>
        <title>{} - Fronting Status</title>
        <style>
            /* generated with ChatGPT o3 */
            /* --- layout container ------------------------------------ */
            body{{
                margin:0;
                padding:1rem;
                font-family:sans-serif;
                display:flex;
                flex-direction: column;
                gap:1rem;
            }}

            /* --- one card -------------------------------------------- */
            body>div {{
                flex:1 1 calc(25% - 1rem);   /* ≤4 cards per row */
                display:flex;
                align-items:center;
                gap:.75rem;
                padding:.75rem;
                background:#fff;
                border-radius:.5rem;
                box-shadow:0 2px 4px rgba(0,0,0,.08);
            }}

            /* --- avatar image ---------------------------------------- */
            body>div img {{
                width:10rem;
                height:10rem;           /* fixed square keeps things tidy */
                object-fit:cover;
                border-radius:50%;
            }}

            /* --- name ------------------------------------------------- */
            body>div p {{
                margin:0;
                font-size: 3rem;
                font-weight:600;
            }}

            /* --- phones & tablets ------------------------------------ */
            @media (max-width:800px) {{
                body>div {{flex:1 1 calc(50% - 1rem);}}   /* 2-across */
            }}
            @media (max-width:420px) {{
                body>div {{flex:1 1 100%;}}               /* stack */
            }}
        </style>
    </head>
    <body>
        {}
    </body>
</html>",
        html_escape::encode_text(&config.system_name),
        fronts_formatted
    )
}
#[post("/register", data = "<credentials>")]
async fn register(
    db_pool: &State<PgPool>,
    credentials: Json<UserCredentials>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(credentials.password.as_bytes(), &salt)
        .map_err(|e| response::Debug(anyhow!(e)))?
        .to_string();
    database::create_user(db_pool, &credentials.username, &hashed_password)
        .await
        .map_err(response::Debug)
}

#[post("/login", data = "<credentials>")]
async fn login(
    db_pool: &State<PgPool>,
    config: &State<Config>,
    credentials: Json<UserCredentials>,
) -> Result<Json<String>, response::Debug<anyhow::Error>> {
    let user = database::get_user(db_pool, &credentials.username)
        .await
        .map_err(response::Debug)?;

    PasswordHash::new(&user.password_hash)
        .map_err(|e| anyhow::anyhow!(e))
        .and_then(|pwh| {
            Argon2::default()
                .verify_password(credentials.password.as_bytes(), &pwh)
                .map_err(|e| anyhow::anyhow!(e))?;
            // todo. get jwt secret from db
            let token = jwt::create_token(user.id, "")?;
            Ok(Json(token))
        })
        .map_err(|_| response::Debug(anyhow!("Invalid credentials")))
}

#[get("/config")]
async fn get_config(
    db_pool: &State<PgPool>,
    token: Result<jwt::Claims>,
) -> Result<Json<config_store::LocalJsonConfigV2>, response::Debug<anyhow::Error>> {
    let claims = token.map_err(response::Debug)?;

    // use db_pool and claims.sub instead and get user specific config
    let config = config_store::read_local_config_file(&CliArgs {
        config: String::new(),
        database_url: String::new(),
    })
    .map_err(response::Debug)?;

    Ok(Json(config))
}

#[post("/config", data = "<config>")]
async fn set_config(
    db_pool: &State<PgPool>,
    token: Result<jwt::Claims>,
    config: Json<config_store::LocalJsonConfigV2>,
) -> Result<(), response::Debug<anyhow::Error>> {
    let claims = token.map_err(response::Debug)?;
    // use this: db_pool, claims.sub
    config_store::write_local_config_file(
        &config.into_inner(),
        &CliArgs {
            config: String::new(),
            database_url: String::new(),
        },
    )
    .map_err(response::Debug)
}
