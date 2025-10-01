use serde::{Deserialize, Serialize};
use strum_macros;

// NOTE: specta::Type is manually exported in bindings
#[derive(
    Clone,
    Serialize,
    Deserialize,
    strum_macros::Display,
    Debug,
    strum_macros::IntoStaticStr,
    strum_macros::VariantNames,
)]
pub enum UpdaterStatus {
    /// User has not enabled this updater in the settings.
    Disabled,
    /// User has enabled this updater and the last update ran successfully.
    Running,
    /// User has enabled this updater and the last update failed with an error.
    Error(String),
    /// User has just enabled this updater and it's not known yet, if it's confirmed to be running successfully.
    Starting,
}
