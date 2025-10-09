fn main() {
    let version = std::env::var("GIT_TAG").unwrap_or_else(|_| "dev".to_string());
    println!("cargo:rustc-env=SP2ANY_VERSION={version}");

    let assets_url = format!(
        "https://github.com/GollyTicker/simply-plural-to-any-updater/releases/tag/{version}"
    );
    println!("cargo:rustc-env=SP2ANY_GITHUB_REPOSITORY_RELEASE_ASSETS_URL={assets_url}");
}