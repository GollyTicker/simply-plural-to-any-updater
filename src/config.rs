use anyhow::Result;
use clap::{self, Parser};
use reqwest::Client;
use std::env::{self, var};
use std::path::Path;
use std::process;
use std::{fs, time};
use time::Duration;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Run without the graphical user interface
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub no_gui: bool,

    // Run in webserver mode. Implies no_gui.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub webserver: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub client: Client,
    pub wait_seconds: Duration,
    pub system_name: String,
    pub simply_plural_token: String,
    pub simply_plural_base_url: String,
    pub vrchat_username: String,
    pub vrchat_password: String,
    pub vrchat_updater_prefix: String,
    pub vrchat_updater_no_fronts: String,
    pub vrchat_updater_truncate_names_to: usize,
    pub vrchat_cookie: String,
}

const DEFAULTS_ENV_URL: &str =
    "https://raw.githubusercontent.com/GollyTicker/simply-plural-to-any-updater/main/defaults.v2.env";
const VRCUPDATER_SAMPLE_ENV_URL: &str = "https://raw.githubusercontent.com/GollyTicker/simply-plural-to-any-updater/main/vrcupdater.sample.env";
const VRC_ENV_PATH_STR: &str = "vrcupdater.env";

pub async fn setup_and_load_config(cli_args: &CliArgs) -> Result<Config> {
    let client = Client::builder().cookie_store(true).build()?;

    // This will run the VRChat updater specific setup if SERVE_API is not already 'true'.
    // It might exit if it creates a sample .env file for the user to edit.
    initialize_environment_for_updater(&client).await?;

    eprintln!("Loading environment variables...");
    let system_name = if cli_args.webserver {
        var("SYSTEM_PUBLIC_NAME")?
    } else {
        String::new()
    };
    let wait_seconds_uint = var("SECONDS_BETWEEN_UPDATES")?.parse::<u64>()?;
    let wait_seconds = Duration::from_secs(wait_seconds_uint);

    let simply_plural_token = var("SPS_API_TOKEN")?;
    let simply_plural_base_url = var("SPS_API_BASE_URL")?;
    eprintln!("Using Simply Plural Base URL: {simply_plural_base_url}");

    let optional_vrchat_config = if cli_args.webserver {
        Ok(String::new())
    } else {
        Err("VRChat variables needs configuration.")
    };

    let vrchat_username = optional_vrchat_config
        .clone()
        .or_else(|_| var("VRCHAT_USERNAME"))?;
    let vrchat_password = optional_vrchat_config
        .clone()
        .or_else(|_| var("VRCHAT_PASSWORD"))?;
    eprintln!("Credentials loaded. VRCHAT_USERNAME is {vrchat_username}");
    let vrchat_cookie = var("VRCHAT_COOKIE").unwrap_or_default();
    if !vrchat_cookie.is_empty() {
        eprintln!("A VRChat cookie was found and will be used.");
    }
    let vrchat_updater_prefix = var("VRCHAT_UPDATER_PREFIX")?;
    let vrchat_updater_no_fronts = var("VRCHAT_UPDATER_NO_FRONTS")?;
    let vrchat_updater_truncate_names_to =
        var("VRCHAT_UPDATER_TRUNCATE_NAMES_TO")?.parse::<usize>()?;

    Ok(Config {
        client,
        wait_seconds,
        system_name,
        simply_plural_token,
        simply_plural_base_url,
        vrchat_username,
        vrchat_password,
        vrchat_updater_prefix,
        vrchat_updater_no_fronts,
        vrchat_updater_truncate_names_to,
        vrchat_cookie,
    })
}

/// Sets up environment variables based on remote and local files for `VRChat` updater mode.
pub async fn initialize_environment_for_updater(client: &Client) -> Result<()> {
    // Only run this setup if SERVE_API is not explicitly true from the environment already
    let serve_api = str::eq(
        &var("SERVE_API").unwrap_or_else(|_| "false".to_owned()),
        "true",
    );
    if serve_api {
        eprintln!("SERVE_API is true, skipping VRChat updater specific environment setup.");
        return Ok(());
    }
    eprintln!("Running VRChat updater specific environment setup...");

    load_remote_defaults_env(client).await?;

    load_vrcupdater_env_or_create_for_user_and_exit(client).await?;

    eprintln!("VRChat updater environment setup complete.");
    Ok(())
}

/// Loads default environment variables from the remote defaults.env file.
/// This is part of the `VRChat` updater specific environment setup.
async fn load_remote_defaults_env(client: &Client) -> Result<()> {
    let defaults_env_content = download_file_content_for_setup(DEFAULTS_ENV_URL, client).await?;
    load_env_vars_from_string(&defaults_env_content, "defaults.env (remote)");
    Ok(())
}

/// Handles the local vrcupdater.env file for `VRChat` updater mode.
/// If it exists, it's loaded. If not, it's created from a remote sample,
/// and the program exits, prompting the user to configure it.
async fn load_vrcupdater_env_or_create_for_user_and_exit(client: &Client) -> Result<()> {
    let vrc_env_path = Path::new(VRC_ENV_PATH_STR);

    if vrc_env_path.exists() {
        let content = fs::read_to_string(vrc_env_path)?;
        load_env_vars_from_string(&content, VRC_ENV_PATH_STR);
        eprintln!("Using local {VRC_ENV_PATH_STR}...");
    } else {
        eprintln!("{VRC_ENV_PATH_STR} not found. Creating sample environment file...");
        let sample_content =
            download_file_content_for_setup(VRCUPDATER_SAMPLE_ENV_URL, client).await?;
        fs::write(vrc_env_path, sample_content)?;
        eprintln!(
            "\n\n\n######### IMPORTANT #########\n\
            Configuration file '{VRC_ENV_PATH_STR}' has been created.\n\
            Please edit it with a simple text editor and\n\
            enter the SimplyPlural and VRChat credentials.\n\
            The file explains how to get these.\n\
            Please, run the application again then."
        );
        process::exit(0); // Exit successfully for user to configure
    }
    Ok(())
}

// Helper function to parse a string containing KEY=VALUE pairs and set them as environment variables.
fn load_env_vars_from_string(content: &str, source_name: &str) {
    eprintln!("Loading environment variables from {source_name} ...");
    for item in dotenvy::Iter::new(content.as_bytes()).filter_map(Result::ok) {
        env::set_var(item.0.clone(), item.1.clone());
    }
}

// Function to store the VRChat Cookie into the .env file of configs.
// If a line starting with VRC_COOKIE is defined in the file,
// then we replace it with VRC_COOKIE="cookie_str".
// Otherwise, we add such a line to the very end and save the file.
pub async fn store_vrchat_cookie(cookie_str: &str) -> Result<()> {
    let vrc_env_path = Path::new(VRC_ENV_PATH_STR);
    let content = fs::read_to_string(vrc_env_path)?;
    let new_cookie_line = format!("VRCHAT_COOKIE=\"{cookie_str}\"");
    let cookie_key_prefix = "VRCHAT_COOKIE=";

    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    if let Some(existing_line_idx) = lines
        .iter()
        .position(|line| line.trim_start().starts_with(cookie_key_prefix))
    {
        lines[existing_line_idx] = new_cookie_line;
    } else {
        lines.push(String::new());
        lines.push("# DO NOT EDIT THE COOKIE BELOW!".to_string());
        lines.push(new_cookie_line);
        lines.push(String::new());
    }

    let new_content = lines.join("\n");
    fs::write(vrc_env_path, new_content)?;

    eprintln!("VRChat cookie stored in {VRC_ENV_PATH_STR}.");
    Ok(())
}

// Helper function to download file content specifically for setup
async fn download_file_content_for_setup(url: &str, client: &reqwest::Client) -> Result<String> {
    let response = client.get(url).send().await?.error_for_status()?;
    let content = response.text().await?;
    eprintln!("Downloaded for setup: {url}");
    Ok(content)
}
