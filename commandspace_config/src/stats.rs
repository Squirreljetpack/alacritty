//! Usage statistics.
//!
//! The counters are application state rather than settings, so they are stored next to the
//! clipboard database instead of in the configuration files.

use std::path::Path;

use cba::bo::write_str;
use log::warn;
use serde::{Deserialize, Serialize};

use crate::LOG_TARGET_CONFIG;

/// Usage counters for the application.
#[derive(Deserialize, Serialize, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// Number of terminal windows opened.
    pub count: u8,
}

impl Stats {
    /// Load the statistics, falling back to the default when the file is missing or unreadable.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();

        let result = std::fs::read_to_string(path)
            .map_err(|err| err.to_string())
            .and_then(|contents| toml::from_str(&contents).map_err(|err| err.to_string()));

        match result {
            Ok(stats) => stats,
            Err(err) if path.is_file() => {
                warn!(
                    target: LOG_TARGET_CONFIG,
                    "Failed to load stats from {}: {err}",
                    path.display(),
                );
                Self::default()
            },
            Err(_) => Self::default(),
        }
    }

    /// Save the statistics, ignoring failures to write the file.
    pub fn save(&self, path: impl AsRef<Path>) {
        let Ok(contents) = toml::to_string(self) else { return };

        if let Err(err) = write_str(path.as_ref(), &contents) {
            warn!(
                target: LOG_TARGET_CONFIG,
                "Failed to save stats to {}: {err}",
                path.as_ref().display(),
            );
        }
    }

    /// Record that a terminal window has been opened.
    pub fn record_window(&mut self) {
        self.count = self.count.saturating_add(1);
    }
}
