use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

use alacritty_terminal::tty::Shell;

use super::{
    action::default_key_bindings, bindings::KeyBinding, serde_utils::deserialize_bindings,
};

/// A delta for a point in a 2 dimensional plane.
#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Delta<T: Default> {
    /// Horizontal change.
    pub x: T,
    /// Vertical change.
    pub y: T,
}

/// Keyboard configuration.
#[derive(serde::Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
pub struct Keyboard {
    /// Keybindings.
    #[serde(skip_serializing)]
    pub bindings: KeyBindings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBindings(pub Vec<KeyBinding>);

impl Default for KeyBindings {
    fn default() -> Self {
        Self(default_key_bindings())
    }
}

impl<'de> Deserialize<'de> for KeyBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(deserialize_bindings(deserializer, Self::default().0)?))
    }
}

/// Wrapper around f32 that represents a percentage value between 0.0 and 1.0.
#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Percentage(f32);

impl Default for Percentage {
    fn default() -> Self {
        Percentage(1.0)
    }
}

impl Percentage {
    pub fn new(value: f32) -> Self {
        Percentage(value.clamp(0., 1.))
    }

    pub fn as_f32(self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Percentage::new(f32::deserialize(deserializer)?))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum Program {
    Just(PathBuf),
    WithArgs {
        program: PathBuf,
        #[serde(default)]
        args: Vec<String>,
    },
}

impl Program {
    pub fn program(&self) -> &Path {
        match self {
            Program::Just(program) => program,
            Program::WithArgs { program, .. } => program,
        }
    }

    pub fn args(&self) -> &[String] {
        match self {
            Program::Just(_) => &[],
            Program::WithArgs { args, .. } => args,
        }
    }
}

impl From<Program> for Shell {
    fn from(value: Program) -> Self {
        match value {
            Program::Just(program) => Shell::new(program, Vec::new()),
            Program::WithArgs { program, args } => Shell::new(program, args),
        }
    }
}
