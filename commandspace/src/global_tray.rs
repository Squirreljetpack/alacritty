use std::process::Command;

use crate::{
    config::action::WindowAction,
    event::{Event, EventLoopProxy},
};
use alacritty_terminal::event::Event as TerminalEvent;
use cba::broc::CommandExt;
use commandspace_config::paths::settings_command;
use thiserror::Error;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

#[allow(unused_imports)]
use tray_icon::menu::accelerator::{Accelerator, Code, Modifiers};

pub use tray_icon::TrayIcon as Tray;
pub type MenuIds = [MenuId; 4];

fn menu() -> tray_icon::menu::Result<(Menu, MenuIds)> {
    let menu = Menu::new();

    macro_rules! add {
        ($label:expr) => {{
            let item = MenuItem::new($label, true, None);
            menu.append(&item)?;
            item.into_id()
        }};
        ($label:expr; $modifiers:expr, $key:expr) => {{
            let accel = Accelerator::new(Some($modifiers), $key);
            let item = MenuItem::new($label, true, Some(accel));
            menu.append(&item)?;
            item.into_id()
        }};
    }

    let toggle = add!("toggle");
    let settings = add!("settings");
    let close = add!("close");
    let quit = add!("quit");

    Ok((menu, [toggle, settings, close, quit]))
}

pub fn tray() -> Result<(Tray, MenuIds), TrayError> {
    let icon = load_icon();

    let (menu, ids) = menu()?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("commandspace")
        .with_icon(icon)
        .build()?;

    Ok((tray_icon, ids))
}

pub fn set_handler(event_proxy: EventLoopProxy, ids: MenuIds) {
    log::trace!("Registering Menu Ids: {ids:?}.");
    let [toggle, settings, close, quit] = ids;
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        log::trace!("Received Menu Event {}.", event.id().0);
        if event.id == &toggle {
            let _ = event_proxy.send_event(Event::new(
                crate::event::EventType::Window(WindowAction::ToggleMaximized),
                None,
            ));
        } else if event.id == &settings {
            let mut cmd = Command::new(settings_command());
            cmd._spawn();
            // let _ =
            //     event_proxy.send_event(Event::new(crate::event::EventType::ShowWindow(None), None));
        } else if event.id == &close {
            let _ = event_proxy.send_event(Event::new(
                crate::event::EventType::Terminal(TerminalEvent::Exit),
                None,
            ));
        } else if event.id == &quit {
            let _ = event_proxy.send_event(Event::new(crate::event::EventType::Quit, None));
        }
    }));
}

pub fn init_tray(event_proxy: EventLoopProxy) -> Result<Tray, TrayError> {
    log::info!("Initializing tray");

    #[cfg(target_os = "linux")]
    {
        std::thread::spawn(|| {
            if let Err(e) = gtk::init() {
                log::error!("Failed to initialize gtk: {e}")
            }

            let Ok((_tray, ids)) = tray() else {
                log::error!("Failed to initialize menu");
                return;
            };

            set_handler(event_proxy, ids);

            gtk::main();
        });

        Err(TrayError::Gtk)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let (tray, ids) = tray()?;
        set_handler(event_proxy, ids);

        Ok(tray)
    }
}

// -----------------
#[derive(Debug, Error)]
pub enum TrayError {
    #[error("tray icon error")]
    TrayIcon(#[from] tray_icon::Error),

    #[error("menu error")]
    Menu(#[from] tray_icon::menu::Error),

    #[cfg(target_os = "linux")]
    #[error("initialized with gtk")]
    Gtk,
}

// fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
//     let (icon_rgba, icon_width, icon_height) = {
//         let image = image::open(path).expect("Failed to open icon path").into_rgba8();
//         let (width, height) = image.dimensions();
//         let rgba = image.into_raw();
//         (rgba, width, height)
//     };
//     tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
// }

fn load_icon() -> tray_icon::Icon {
    // Include the PNG bytes at compile time
    const ICON_BYTES: &[u8] = include_bytes!("../extra/logo/compat/alacritty-term.png");

    // Decode the image
    let image = image::load_from_memory(ICON_BYTES).expect("Failed to decode icon").into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon")
}
