use keyboard_types::{Code, Modifiers};

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
pub enum GlobalAction {
    Window(WindowAction),
    Command(CommandAction),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CommandAction {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GlobalBindings(pub GlobalBindingsMap, pub HotKey);

pub type GlobalBindingsMap = Vec<(HotKey, GlobalAction)>;

impl Default for GlobalBindings {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        let toggle = HotKey::new(Modifiers::META, Code::Space);
        #[cfg(not(target_os = "macos"))]
        let toggle = HotKey::new(Modifiers::CONTROL, Code::Space);

        Self(Default::default(), toggle)
    }
}

// impl std::ops::DerefMut for CustomActions {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
