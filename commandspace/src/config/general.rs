//! Miscellaneous configuration options.

use serde::Serialize;

use alacritty_config_derive::ConfigDeserialize;

/// General config section.
///
/// This section is for fields which can not be easily categorized,
/// to avoid common TOML issues with root-level fields.
#[derive(ConfigDeserialize, Serialize, Clone, PartialEq, Debug)]
pub struct General {
    /// Configuration file imports.
    ///
    /// This is never read since the field is directly accessed through the config's
    /// [`toml::Value`], but still present to prevent unused field warnings.
    pub import: Vec<String>,

    /// Live config reload.
    pub live_config_reload: bool,
}

impl Default for General {
    fn default() -> Self {
        Self { live_config_reload: true, import: Default::default() }
    }
}
