//! Usage statistics.
//!
//! The counters are application state rather than settings, so they are stored next to the
//! clipboard database instead of in the configuration files.

use std::collections::BTreeMap;
use std::path::Path;

use cba::bo::write_str;
use chrono::{Duration, Local, NaiveDate};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::LOG_TARGET_CONFIG;

/// How the day keys are formatted, both for writing and for parsing them back.
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Number of past days that are kept in the file.
const KEEP_DAYS: i64 = 365;

/// Usage counters for the application.
#[derive(Deserialize, Serialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Stats {
    /// Number of terminal windows opened since the counters were first written.
    pub total: u32,
    /// Number of terminal windows opened per local date, keyed as `YYYY-MM-DD`.
    pub days: BTreeMap<String, u32>,
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
    ///
    /// The file is replaced in one step so that a reader never sees a half-written file.
    pub fn save(&self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let Ok(contents) = toml::to_string(self) else { return };

        let temp = path.with_extension("tmp");
        if let Err(err) = write_str(&temp, &contents) {
            warn!(
                target: LOG_TARGET_CONFIG,
                "Failed to save stats to {}: {err}",
                path.display(),
            );
            return;
        }

        if let Err(err) = std::fs::rename(&temp, path) {
            warn!(
                target: LOG_TARGET_CONFIG,
                "Failed to replace stats at {}: {err}",
                path.display(),
            );
        }
    }

    /// Record that a terminal window has been opened.
    pub fn record_window(&mut self) {
        let today = Local::now().date_naive();

        self.total = self.total.saturating_add(1);

        let count = self.days.entry(today.format(DATE_FORMAT).to_string()).or_default();
        *count = count.saturating_add(1);

        self.prune(today);
    }

    /// Drop the days that fall outside the retention window.
    fn prune(&mut self, today: NaiveDate) {
        let oldest = today - Duration::days(KEEP_DAYS - 1);

        self.days.retain(|day, _| {
            NaiveDate::parse_from_str(day, DATE_FORMAT).map(|date| date >= oldest).unwrap_or(false)
        });
    }
}
