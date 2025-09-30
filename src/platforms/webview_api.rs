use crate::communication::HttpResult;
use crate::database;
use crate::plurality;
use crate::users;
use rocket::{
    State,
    response::{self, content::RawHtml},
};
use sqlx::PgPool;

#[get("/fronting/<website_url_name>")]
pub async fn get_api_fronting_by_user_id(
    website_url_name: &str,
    db_pool: &State<PgPool>,
    application_user_secrets: &State<database::ApplicationUserSecrets>,
    client: &State<reqwest::Client>,
) -> HttpResult<RawHtml<String>> {
    log::info!("# | GET /fronting/{website_url_name}");

    let user_info = database::find_user_by_website_url_name(db_pool, website_url_name).await?;
    let user_id = user_info.id;

    log::info!("# | GET /fronting/{website_url_name} | {user_id}");

    let user_config =
        database::get_user_secrets(db_pool, &user_id, application_user_secrets).await?;

    let (updater_config, _) =
        users::create_config_with_strong_constraints(&user_id, client, &user_config)?;

    log::info!("# | GET /fronting/{website_url_name} | {user_id} | got_config");

    let fronts = plurality::fetch_fronts(&updater_config)
        .await
        .map_err(response::Debug)?;

    log::info!("# | GET /fronting/{website_url_name} | {user_id} | got_config | fetched_fronts");

    let html = generate_html(&updater_config.website_system_name, fronts);

    log::info!(
        "# | GET /fronting/{website_url_name} | {user_id} | got_config | fetched_fronts | HTML generated"
    );

    Ok(RawHtml(html))
}

fn generate_html(website_system_name: &str, fronts: Vec<plurality::Fronter>) -> String {
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
        html_escape::encode_text(website_system_name),
        fronts_formatted
    )
}
