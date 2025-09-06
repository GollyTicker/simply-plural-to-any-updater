use std::env;

use sp2any::users;

#[tauri::command]
async fn login(email: String, password: String) -> Result<users::JwtString, String> {
    let login_creds = users::UserLoginCredentials {
        email: email.into(),
        password: users::UserProvidedPassword { inner: password },
    };

    let client = reqwest::Client::new();
    let base_url = env::var("SP2ANY_BASE_URL").map_err(|e| e.to_string())?;
    let login_url = format!("{}{}", base_url, "/api/user/login");

    let res = client
        .post(login_url)
        .json(&login_creds)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        let login_response = res
            .json::<users::JwtString>()
            .await
            .map_err(|e| e.to_string())?;
        Ok(login_response)
    } else {
        Err(format!("Login failed: {}", res.status()))
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![login])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Bridge Startup error.");
}
