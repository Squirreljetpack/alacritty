pub mod paths;

pub mod action;
pub mod bell;
pub mod bg;
pub mod color;
pub mod configs;
pub mod cursor;
pub mod debug;
pub mod font;
pub mod hint;
pub mod rgb;
pub mod scrolling;
pub mod selection;
pub mod serde_utils;
pub mod terminal;
pub mod types;
pub mod window;

mod bindings;
pub mod global_bindings;
mod mouse;

pub use action::{Action, MouseAction, SearchAction, ViAction};
// #[cfg(test)]
pub use bindings::Binding;
pub use bindings::{BindingKey, BindingMode, KeyBinding, MouseEvent};
pub use configs::{AlacrittyConfig, Config};

/// Logging target for config error messages.
pub const LOG_TARGET_CONFIG: &str = "commandspace_config_derive";
