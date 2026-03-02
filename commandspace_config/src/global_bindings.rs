#![allow(unused)]

use keyboard_types::{Code, Modifiers};
use std::collections::HashMap;

use crate::action::WindowAction;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GlobalBinding {
    action: GlobalAction,
    #[serde(flatten)]
    hotkey: HotKey,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HotKey {
    pub mods: Modifiers,
    // #[cfg(not(target_os = "macos"))]
    pub key: Code,
}

impl HotKey {
    #[inline]
    pub const fn new(mods: Modifiers, key: Code) -> Self {
        Self { mods, key }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GlobalAction {
    Window(WindowAction),
    Command(CommandAction),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CommandAction {
    command: String,
    args: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct GlobalBindings(pub Vec<(HotKey, GlobalAction)>);

impl GlobalBindings {
    pub fn default_binds() -> Self {
        #[cfg(target_os = "macos")]
        let toggle = HotKey::new(Modifiers::META, Code::Space);
        #[cfg(not(target_os = "macos"))]
        let toggle = HotKey::new(Modifiers::CONTROL, Code::Space);
        let close = HotKey::new(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyW);
        let ret = vec![(toggle, GlobalAction::Window(WindowAction::Toggle))];
        Self(ret)
    }
}
// impl std::ops::DerefMut for CustomActions {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
