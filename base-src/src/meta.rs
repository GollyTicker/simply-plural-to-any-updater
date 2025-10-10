use serde::{Deserialize, Serialize};

pub const CANONICAL_SP2ANY_BASE_URL: &str = "https://public-test.sp2any.ayake.net";

pub const SP2ANY_GITHUB_REPOSITORY_URL: &str =
    "https://github.com/GollyTicker/simply-plural-to-any-updater";

pub const SP2ANY_VERSION: &str = env!("SP2ANY_VERSION");

pub const SP2ANY_GITHUB_REPOSITORY_RELEASES_URL: &str =
    "https://github.com/GollyTicker/simply-plural-to-any-updater/releases";

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SP2AnyVariantInfo {
    pub version: String,
    pub variant: String,
    pub description: Option<String>,
    pub show_in_ui: bool,
}
