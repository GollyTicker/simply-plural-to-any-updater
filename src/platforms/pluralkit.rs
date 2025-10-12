use crate::{
    int_counter_metric, metric, plurality, record_if_error, users, users::UserConfigForUpdater,
};
use anyhow::Result;

int_counter_metric!(PLURAKIT_API_REQUESTS_TOTAL);
metric!(
    rocket_prometheus::prometheus::IntGaugeVec,
    PLURAKIT_API_RATELIMIT_REMAINING,
    "plurakit_api_ratelimit_remaining",
    &["user_id", "scope"]
);

const PLURALKIT_UPDATER_USER_AGENT: &str =
    concat!("SP2Any/", env!("CARGO_PKG_VERSION"), " Discord: .ay", "ake");

pub struct PluralKitUpdater {
    pub last_operation_error: Option<String>,
}

impl Default for PluralKitUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl PluralKitUpdater {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_operation_error: None,
        }
    }

    #[allow(clippy::unused_async)]
    pub async fn setup(&mut self, _config: &users::UserConfigForUpdater) -> Result<()> {
        // Nothing to do here for now
        Ok(())
    }

    pub async fn update_fronting_status(
        &mut self,
        config: &users::UserConfigForUpdater,
        fronts: &[plurality::Fronter],
    ) -> Result<()> {
        record_if_error!(self, update_to_pluralkit(config, fronts).await)
    }
}

async fn update_to_pluralkit(
    config: &UserConfigForUpdater,
    fronts: &[plurality::Fronter],
) -> Result<()> {
    let pluralkit_ids: Vec<&str> = fronts
        .iter()
        .filter_map(|f| f.pluralkit_id.as_ref())
        .map(std::string::String::as_str)
        .collect();

    let request_body = serde_json::json!({ "members": pluralkit_ids });

    PLURAKIT_API_REQUESTS_TOTAL
        .with_label_values(&[&config.user_id.to_string()])
        .inc();

    let response = config
        .client
        .post("https://api.pluralkit.me/v2/systems/@me/switches")
        .header("Authorization", &config.pluralkit_token.secret)
        .header("Content-Type", "application/json")
        .header("User-Agent", PLURALKIT_UPDATER_USER_AGENT)
        .json(&request_body)
        .send()
        .await?
        .error_for_status()?;

    measure_rate_limits(config, &response);

    Ok(())
}

fn measure_rate_limits(config: &UserConfigForUpdater, response: &reqwest::Response) {
    let headers = response.headers();
    let rate_limit_limit = headers
        .get("X-RateLimit-Limit")
        .and_then(|v| v.to_str().ok());
    let rate_limit_remaining = headers
        .get("X-RateLimit-Remaining")
        .and_then(|v| v.to_str().ok().and_then(|s| s.parse().ok()));
    let rate_limit_reset = headers
        .get("X-RateLimit-Reset")
        .and_then(|v| v.to_str().ok());
    let rate_limit_scope = headers
        .get("X-RateLimit-Scope")
        .and_then(|v| v.to_str().ok());

    if let (Some(remaining), Some(scope)) = (rate_limit_remaining, rate_limit_scope) {
        PLURAKIT_API_RATELIMIT_REMAINING
            .with_label_values(&[&config.user_id.to_string(), scope])
            .set(remaining);
    }

    log::info!(
        "# | update_to_pluralkit | {} | updated | rate limit: limit={:?}, remaining={:?}, reset={:?}, scope={:?}",
        config.user_id,
        rate_limit_limit,
        rate_limit_remaining,
        rate_limit_reset,
        rate_limit_scope
    );
}
