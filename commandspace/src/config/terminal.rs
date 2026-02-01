use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize, de};
use toml::Value;

use alacritty_terminal::term::Osc52;

use super::types::Program;

use super::serde_utils::StringVisitor;

#[derive(serde::Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
pub struct Terminal {
    /// OSC52 support mode.
    pub osc52: SerdeOsc52,
    /// Path to a shell program to run on startup.
    pub shell: Option<Program>,

    /// Default working directory.
    pub working_directory: Option<PathBuf>,
}

#[derive(Serialize, Default, Copy, Clone, Debug, PartialEq)]
pub struct SerdeOsc52(pub Osc52);

impl<'de> Deserialize<'de> for SerdeOsc52 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = deserializer.deserialize_str(StringVisitor)?;
        Osc52::deserialize(Value::String(value)).map(SerdeOsc52).map_err(de::Error::custom)
    }
}
