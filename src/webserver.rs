use crate::config::Config;
use crate::simply_plural::{self};
use anyhow::Result;
use rocket::{
    response::{self, content::RawHtml},
    State,
};

pub async fn run_server(config: &Config) -> Result<()> {
    rocket::build()
        .manage(config.clone()) // Rocket needs the owned or shareable Config
        .mount("/", routes![rest_get_fronting])
        .launch()
        .await
        .map_err(|e| anyhow::anyhow!("Rocket failed with: {}", e))
        .map(|_| ())
}

#[get("/fronting")]
async fn rest_get_fronting(
    config: &State<Config>,
) -> Result<RawHtml<String>, response::Debug<anyhow::Error>> {
    let fronts = simply_plural::fetch_fronts(config.inner())
        .await
        .map_err(|e| response::Debug(e))?; // Convert anyhow::Error to response::Debug
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
        r#"<html>
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
</html>"#,
        html_escape::encode_text(&config.system_name),
        fronts_formatted
    )
}
