fn main() {
    let version = std::env::var("GIT_TAG").unwrap_or_else(|_| "dev".to_string());
    println!("cargo:rustc-env=PLURALSYNC_VERSION={version}");
}
