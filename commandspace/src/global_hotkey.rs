// #[cfg(target_os = "macos")]

use std::{process::Command, time::Instant};

use crate::{
    config::global_bindings::GlobalBindingsMap,
    event::{Event, EventLoopProxy},
};

mod inner {

    use super::*;

    use std::sync::Arc;

    use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};
    use parking_lot::Mutex;
    use tokio::sync::watch;

    pub type BindingsMap = Arc<Mutex<Vec<(u32, GlobalAction)>>>;

    pub fn start_hotkeys_task(
        mut receiver: watch::Receiver<GlobalBindingsMap>,
        event_proxy: EventLoopProxy,
    ) {
        let bindings_map: BindingsMap = Arc::new(Mutex::new(Vec::new()));
        let handler_bindings = Arc::clone(&bindings_map);
        let handler_proxy = event_proxy.clone();

        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if matches!(event.state, HotKeyState::Pressed) {
                log::debug!("Received hotkey {event:?}.");
                let map = handler_bindings.lock();
                if let Some(action) =
                    map.iter().find_map(|(id, action)| (*id == event.id).then_some(action))
                {
                    action.clone().dispatch(&handler_proxy);
                }
            }
        }));

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for hotkeys");

            rt.block_on(async move {
                let mut _current_manager: Option<GlobalHotKeyManager> = None;

                loop {
                    let bindings = receiver.borrow_and_update().clone();

                    match init_hotkeys_internal(bindings) {
                        Ok((manager, map)) => {
                            _current_manager = Some(manager);
                            *bindings_map.lock() = map;
                            log::info!("Global hotkeys updated successfully");
                        },
                        Err(e) => {
                            log::error!("Failed to initialize hotkeys: {:?}", e);
                        },
                    }

                    if receiver.changed().await.is_err() {
                        break;
                    }
                }
            });
        });
    }

    fn init_hotkeys_internal(
        bindings: GlobalBindingsMap,
    ) -> global_hotkey::Result<(GlobalHotKeyManager, Vec<(u32, GlobalAction)>)> {
        let hotkeys_manager = GlobalHotKeyManager::new()?;
        let mut hk_binds = Vec::new();

        for (hotkey, action) in bindings {
            let mods = hotkey.mods;
            let code = hotkey.key;
            let hk = HotKey::new(Some(mods), code);
            hotkeys_manager.register(hk)?;
            hk_binds.push((hk.id(), action));
        }

        Ok((hotkeys_manager, hk_binds))
    }

    // pub fn init_hotkeys(
    //     bindings: GlobalBindingsMap,
    //     event_proxy: EventLoopProxy,
    // ) -> global_hotkey::Result<GlobalHotKeyManager> {
    //     let (manager, hk_binds) = init_hotkeys_internal(bindings)?;
    //     let hk_binds = Arc::new(Mutex::new(hk_binds));

    //     GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
    //         if matches!(event.state, HotKeyState::Pressed) {
    //             log::debug!("Recieved hotkey {event:?}.");
    //             _dbg!(event);
    //             let map = hk_binds.lock();
    //             if let Some(action) =
    //                 map.iter().find_map(|(id, action)| (*id == event.id).then_some(action))
    //             {
    //                 action.dispatch(&event_proxy);
    //             }
    //         }
    //     }));

    //     Ok(manager)
    // }
}

pub static LOST_FOCUS: Mutex<Option<Instant>> = Mutex::new(None);

use crate::config::global_bindings::GlobalAction;
use cba::broc::CommandExt;
use easy_ext::ext;

#[ext]
impl GlobalAction {
    fn dispatch(self, event_proxy: &EventLoopProxy) {
        match self {
            GlobalAction::Window(action) => {
                event_proxy.send_event(Event::new(crate::event::EventType::Window(action), None))
            },
            GlobalAction::Command(cmd) => {
                let mut cmd = Command::new(cmd.command).with_args(cmd.args);
                cmd.spawn_detached();
            },
        }
    }
}

// #[cfg(not(target_os = "macos"))]
// mod inner {
//     use crate::{
//         _dbg,
//         config::global_bindings::HotKey,
//         event::{Event, EventLoopProxy},
//     };
//     use rdev::{Event as RdevEvent, EventType, Key, listen};
//     use std::thread::JoinHandle;

//     use winit::keyboard::ModifiersState;

//     #[derive(Debug)]
//     pub enum HotkeyAction {
//         Toggle,
//     }

//     pub fn init_hotkeys(event_proxy: EventLoopProxy) -> Result<JoinHandle<()>, ()> {
//         let hotkeys = {
//             let mut hotkeys = std::collections::HashMap::new();
//             hotkeys.insert(HotKey::new(ModifiersState::CONTROL, Key::Space), HotkeyAction::Toggle);
//             hotkeys
//         };

//         let ret = std::thread::spawn(|| rdev_listen(event_proxy, hotkeys));

//         Ok(ret)
//     }
//     pub type HotkeyMap = std::collections::HashMap<HotKey, HotkeyAction>;

//     pub fn rdev_listen(event_proxy: EventLoopProxy, hotkeys: HotkeyMap) {
//         let mut pressed_mods = Modifiers::empty();

//         let callback = move |event: RdevEvent| match event.event_type {
//             EventType::KeyPress(key) => {
//                 if let Some(m) = modifier_from_key(key) {
//                     pressed_mods |= m;
//                     return;
//                 }
//                 let current = HotKey::new(pressed_mods, key);

//                 if let Some(action) = hotkeys.get(&current) {
//                     _dbg!(action);
//                     log::debug!("Triggered hotkey action: {action:?}");
//                     match action {
//                         HotkeyAction::Toggle => {
//                             let _ = event_proxy.send_event(Event::new(
//                                 crate::event::EventType::ShowWindow(None),
//                                 None,
//                             ));
//                         },
//                     }
//                 }
//             },

//             EventType::KeyRelease(key) => {
//                 if let Some(m) = modifier_from_key(key) {
//                     pressed_mods.remove(m);
//                 }
//             },

//             _ => {},
//         };

//         if let Err(err) = listen(callback) {
//             log::error!("rdev listener failed: {err:?}");
//         }
//     }

//     #[inline]
//     fn modifier_from_key(key: Key) -> Option<ModifiersState> {
//         Some(match key {
//             Key::ControlLeft | Key::ControlRight => ModifiersState::CONTROL,
//             Key::ShiftLeft | Key::ShiftRight => ModifiersState::SHIFT,
//             Key::Alt => ModifiersState::ALT,
//             Key::MetaLeft | Key::MetaRight => ModifiersState::META,
//             _ => return None,
//         })
//     }
// }

pub use inner::*;
use parking_lot::Mutex;
