#[cfg(target_os = "macos")]
mod inner {
    use crate::{
        _dbg,
        event::{Event, EventLoopProxy},
    };
    use global_hotkey::{
        GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
        hotkey::{Code, HotKey, Modifiers},
    };

    pub fn init_hotkeys(event_proxy: EventLoopProxy) -> global_hotkey::Result<GlobalHotKeyManager> {
        let hotkeys_manager = GlobalHotKeyManager::new()?;

        macro_rules! hk {
            ($mods:expr, $code:expr) => {{
                let hk = HotKey::new(Some($mods), $code);
                hotkeys_manager.register(hk)?;
                hk.id()
            }};
            ($code:expr) => {{
                let hk = HotKey::new(None, $code);
                hotkeys_manager.register(hk)?;
                hk.id()
            }};
        }

        let toggle = hk!(Modifiers::CONTROL, Code::Space);

        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if matches!(event.state, HotKeyState::Pressed) {
                log::debug!("Recieved hotkey {event:?}.");
                _dbg!(event);

                if event.id == toggle {
                    let _ = event_proxy
                        .send_event(Event::new(crate::event::EventType::ShowWindow(None), None));
                }
            }
        }));

        Ok(hotkeys_manager)
    }
}

#[cfg(not(target_os = "macos"))]
mod inner {
    use crate::{
        _dbg,
        event::{Event, EventLoopProxy},
    };
    use rdev::{Event as RdevEvent, EventType, Key, listen};
    use std::thread::JoinHandle;

    use winit::keyboard::ModifiersState;
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HotKey {
        pub mods: ModifiersState,
        pub key: Key,
    }

    impl HotKey {
        #[inline]
        pub const fn new(mods: ModifiersState, key: Key) -> Self {
            Self { mods, key }
        }
    }

    #[derive(Debug)]
    pub enum HotkeyAction {
        Toggle,
    }

    pub fn init_hotkeys(event_proxy: EventLoopProxy) -> Result<JoinHandle<()>, ()> {
        let hotkeys = {
            let mut hotkeys = std::collections::HashMap::new();
            hotkeys.insert(HotKey::new(ModifiersState::CONTROL, Key::Space), HotkeyAction::Toggle);
            hotkeys
        };

        let ret = std::thread::spawn(|| rdev_listen(event_proxy, hotkeys));

        Ok(ret)
    }
    pub type HotkeyMap = std::collections::HashMap<HotKey, HotkeyAction>;

    pub fn rdev_listen(event_proxy: EventLoopProxy, hotkeys: HotkeyMap) {
        let mut pressed_mods = ModifiersState::empty();

        let callback = move |event: RdevEvent| match event.event_type {
            EventType::KeyPress(key) => {
                if let Some(m) = modifier_from_key(key) {
                    pressed_mods |= m;
                    return;
                }
                let current = HotKey::new(pressed_mods, key);

                if let Some(action) = hotkeys.get(&current) {
                    _dbg!(action);
                    log::debug!("Triggered hotkey action: {action:?}");
                    match action {
                        HotkeyAction::Toggle => {
                            let _ = event_proxy.send_event(Event::new(
                                crate::event::EventType::ShowWindow(None),
                                None,
                            ));
                        },
                    }
                }
            },

            EventType::KeyRelease(key) => {
                if let Some(m) = modifier_from_key(key) {
                    pressed_mods.remove(m);
                }
            },

            _ => {},
        };

        if let Err(err) = listen(callback) {
            log::error!("rdev listener failed: {err:?}");
        }
    }

    #[inline]
    fn modifier_from_key(key: Key) -> Option<ModifiersState> {
        Some(match key {
            Key::ControlLeft | Key::ControlRight => ModifiersState::CONTROL,
            Key::ShiftLeft | Key::ShiftRight => ModifiersState::SHIFT,
            Key::Alt => ModifiersState::ALT,
            Key::MetaLeft | Key::MetaRight => ModifiersState::META,
            _ => return None,
        })
    }
}

pub use inner::*;