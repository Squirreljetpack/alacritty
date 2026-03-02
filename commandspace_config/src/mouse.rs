use serde::{Deserialize, Deserializer, Serialize};

use super::{
    action::default_mouse_bindings, bindings::MouseBinding, serde_utils::deserialize_bindings,
};

#[derive(serde::Deserialize, Serialize, Default, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct Mouse {
    pub hide_when_typing: bool,
    #[serde(skip_serializing)]
    pub bindings: MouseBindings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MouseBindings(pub Vec<MouseBinding>);

impl Default for MouseBindings {
    fn default() -> Self {
        Self(default_mouse_bindings())
    }
}

impl<'de> Deserialize<'de> for MouseBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self(deserialize_bindings(deserializer, Self::default().0)?))
    }
}
