use crate::auth;
use crate::config::Config;
use crate::config_store;
use crate::database;
use crate::model::ApplicationJwtSecret;
use crate::model::JwtString;
use crate::model::UserLoginCredentials;
use crate::simply_plural;
use crate::updater::UpdaterState;
use crate::CliArgs;
use anyhow::{anyhow, Result};
use rocket::{
    response::{self, content::RawHtml},
    serde::json::Json,
    State,
};
use sqlx::PgPool;

pub async fn run_server(cli_args: CliArgs, config: Config, db_pool: PgPool) -> Result<()> {
    rocket::build()
        .manage(config)
        .manage(cli_args)
        .manage(db_pool)
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
    config: &State<Config>,
    credentials: Json<UserLoginCredentials>,
) -> Result<Json<JwtString>, response::Debug<anyhow::Error>> {
    let db_user = database::get_user(db_pool, credentials.email.clone())
        .await
        .map_err(response::Debug)?;

    let jwt_secret = ApplicationJwtSecret {
        inner: String::from("todo-get-this-secret-from-env-during-startup"),
    };

    let jwt_string =
        auth::verify_password_and_create_token(credentials.password.clone(), db_user, jwt_secret)?;

    Ok(Json(jwt_string))
}
// todo. how can we enable users to reset their password? Do I really have to do this all manually here???

#[get("/user/config")]
async fn get_config(
    db_pool: &State<PgPool>,
    token: Result<JwtString>,
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

#[post("/user/config", data = "<config>")]
async fn set_config(
    db_pool: &State<PgPool>,
    token: Result<JwtString>,
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
