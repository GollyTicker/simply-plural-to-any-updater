use std::path::Path;

fn main() {
    // sqlx: trigger recompilation when a new migration is added or the license text has changed
    println!("cargo:rerun-if-changed=docker/migrations");
    println!("cargo:rerun-if-changed=docker/license*");

    // expose env variables from `test/secrets.env` to the build process
    if Path::new("test/secrets.env").exists() {
        dotenvy::from_path("test/secrets.env").ok();
        for var in ["USER_AGENT_EMAIL", "USER_AGENT_DISCORD_USERNAME"] {
            if let Ok(value) = std::env::var(var) {
                println!("cargo:rustc-env={}={}", var, value);
            }
        }
    }
}
