//! Usage statistics.
//!
//! The counters are application state rather than settings, so they are stored next to the
//! clipboard database instead of in the configuration files.

use std::collections::BTreeMap;
use std::path::Path;

use cba::bait::ResultExt;
use cba::bo::{dump_type, load_type_or_default_log};
use jiff::civil::Date;
use jiff::{Span, Zoned};
use serde::{Deserialize, Serialize};

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
        load_type_or_default_log(path, |contents| toml::from_str(contents))
    }

    /// Save the statistics, ignoring failures to write the file.
    ///
    /// The data directory is created when the terminal starts, so the file always has a parent here.
    pub fn save(&self, path: impl AsRef<Path>) {
        dump_type(path, self, toml::to_string)._wlog();
    }

    /// Record that a terminal window has been opened.
    pub fn record_window(&mut self) {
        let today = Zoned::now().date();

        self.total = self.total.saturating_add(1);

        let count = self.days.entry(today.strftime(DATE_FORMAT).to_string()).or_default();
        *count = count.saturating_add(1);

        self.prune(today);
    }

    /// Drop the days that fall outside the retention window.
    fn prune(&mut self, today: Date) {
        let oldest = today.checked_sub(Span::new().days(KEEP_DAYS - 1)).unwrap_or(Date::MIN);

        self.days.retain(|day, _| {
            Date::strptime(DATE_FORMAT, day).map(|date| date >= oldest).unwrap_or(false)
        });
    }
}
