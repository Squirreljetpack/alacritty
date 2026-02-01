use std::time::Duration;

use serde::Serialize;

use super::rgb::Rgb;
use super::types::Program;

#[derive(serde::Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct BellConfig {
    /// Visual bell animation function.
    pub animation: BellAnimation,

    /// Command to run on bell.
    pub command: Option<Program>,

    /// Visual bell flash color.
    pub color: Rgb,

    /// Visual bell duration in milliseconds.
    duration: u16,
}

impl Default for BellConfig {
    fn default() -> Self {
        Self {
            color: Rgb::new(255, 255, 255),
            animation: Default::default(),
            command: Default::default(),
            duration: Default::default(),
        }
    }
}

impl BellConfig {
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.duration as u64)
    }
}

/// `VisualBellAnimations` are modeled after a subset of CSS transitions and Robert
/// Penner's Easing Functions.
#[derive(serde::Deserialize, Serialize, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BellAnimation {
    // CSS animation.
    Ease,
    // CSS animation.
    EaseOut,
    // Penner animation.
    EaseOutSine,
    // Penner animation.
    EaseOutQuad,
    // Penner animation.
    EaseOutCubic,
    // Penner animation.
    EaseOutQuart,
    // Penner animation.
    EaseOutQuint,
    // Penner animation.
    EaseOutExpo,
    // Penner animation.
    EaseOutCirc,
    // Penner animation.
    #[default]
    Linear,
}
