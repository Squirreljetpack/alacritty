use serde::Serialize;

use alacritty_terminal::term::SEMANTIC_ESCAPE_CHARS;

#[derive(serde::Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct Selection {
    pub semantic_escape_chars: String,
    pub save_to_clipboard: bool,
}

impl Default for Selection {
    fn default() -> Self {
        Self {
            semantic_escape_chars: SEMANTIC_ESCAPE_CHARS.to_owned(),
            save_to_clipboard: Default::default(),
        }
    }
}
