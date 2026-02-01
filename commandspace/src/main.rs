//! Alacritty - The GPU Enhanced Terminal.

#![warn(rust_2018_idioms, future_incompatible)]
#![deny(clippy::all, clippy::if_not_else, clippy::enum_glob_use)]
#![cfg_attr(clippy, deny(warnings))]
// With the default subsystem, 'console', windows creates an additional console
// window for the program.
// This is silently ignored on non-windows systems.
// See https://msdn.microsoft.com/en-us/library/4cc7ya5b.aspx for more details.
#![windows_subsystem = "windows"]

#[cfg(not(any(feature = "x11", feature = "wayland", target_os = "macos", windows)))]
compile_error!(r#"at least one of the "x11"/"wayland" features must be enabled"#);

use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;
use std::{env, fs};

use cli_boilerplate_automation::bait::ResultExt;
use cli_boilerplate_automation::bog;
use log::info;
#[cfg(windows)]
use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
use winit::event_loop::EventLoop;
#[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};

use alacritty_terminal::tty;

mod cli;
mod clipboard;
mod config;
mod daemon;
mod display;
mod event;
mod input;
mod logging;
#[cfg(target_os = "macos")]
mod macos;
mod message_bar;
#[cfg(windows)]
mod panic;
// mod rdev;
mod renderer;
mod scheduler;
mod string;
mod window_context;

mod fzl;
mod global_hotkey;
mod paths;
mod tray;
mod utils;

mod config_monitor;

mod gl {
    #![allow(clippy::all, unsafe_op_in_unsafe_fn)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use crate::cli::Options;
use crate::config::global_bindings::GlobalBindings;
use crate::config_monitor::ConfigMonitor;
use crate::event::{EventLoopProxy, Processor};
#[cfg(target_os = "macos")]
use crate::macos::locale;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    panic::attach_handler();

    // When linked with the windows subsystem windows won't automatically attach
    // to the console of the parent process, so we do it explicitly. This fails
    // silently if the parent has no console.
    #[cfg(windows)]
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    // Load command line options.
    let options = <Options as clap::Parser>::parse();

    // match options.subcommands {
    //     #[cfg(unix)]
    //     Some(Subcommands::Msg(options)) => msg(options)?,
    //     Some(Subcommands::Migrate(options)) => migrate::migrate(options),
    //     None => alacritty(options)?,
    // }

    alacritty(options)?;

    Ok(())
}

/// `msg` subcommand entrypoint.
// #[cfg(unix)]
// #[allow(unused_mut)]
// fn msg(mut _options: MessageOptions) -> Result<(), Box<dyn Error>> {
//     // unimplemented
//     Ok(())
// }

/// Temporary files stored for Alacritty.
///
/// This stores temporary files to automate their destruction through its `Drop` implementation.
struct TemporaryFiles {
    log_file: Option<PathBuf>,
}

impl Drop for TemporaryFiles {
    fn drop(&mut self) {
        // Clean up logfile.
        if let Some(log_file) = &self.log_file {
            if fs::remove_file(log_file).is_ok() {
                let _ = writeln!(io::stdout(), "Deleted log file at \"{}\"", log_file.display());
            }
        }
    }
}

/// Run main Alacritty entrypoint.
///
/// Creates a window, the terminal state, PTY, I/O event loop, input processor,
/// config change monitor, and runs the main display loop.
fn alacritty(mut options: Options) -> Result<(), Box<dyn Error>> {
    // Setup winit event loop.
    let window_event_loop = EventLoop::new()?;
    let (tx, rx) = std::sync::mpsc::channel();
    let proxy = EventLoopProxy::new(tx, window_event_loop.create_proxy());

    // Initialize the logger as soon as possible as to capture output from other subsystems.
    let log_file =
        logging::initialize(&options, proxy.clone()).expect("Unable to initialize logger");
    bog::init_bogger(true, false);
    bog::init_filter(options.bog_level());

    info!("Welcome to Commandspace");
    info!("Version {}", env!("VERSION"));

    #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
    info!(
        "Running on {}",
        if matches!(
            window_event_loop.display_handle().unwrap().as_raw(),
            RawDisplayHandle::Wayland(_)
        ) {
            "Wayland"
        } else {
            "X11"
        }
    );
    #[cfg(not(any(feature = "x11", target_os = "macos", windows)))]
    info!("Running on Wayland");

    // Load configuration file.
    let (config, _general_cfg) = cli::config::load(&mut options);

    // Update the log level from config.
    log::set_max_level(config.debug.log_level);

    // Set tty environment variables.
    tty::setup_env();

    // Set env vars from config.
    for (key, value) in config.env.iter() {
        unsafe { env::set_var(key, value) };
    }

    // Switch to home directory.
    #[cfg(target_os = "macos")]
    env::set_current_dir(home::home_dir().unwrap()).unwrap();

    // Set macOS locale.
    #[cfg(target_os = "macos")]
    locale::set_locale_environment();

    #[cfg(target_os = "macos")]
    macos::disable_autofill();

    // Setup automatic RAII cleanup for our files.
    let log_cleanup = log_file.filter(|_| !config.debug.persistent_logging);
    let _files = TemporaryFiles { log_file: log_cleanup };

    // hotkey manager
    let _hotkey_manager =
        global_hotkey::init_hotkeys(GlobalBindings::default_binds(), proxy.clone())
            .prefix("Unable to initialize hotkeys")
            ._elog();

    // Event processor.
    let processor = Processor::new(config, options, &window_event_loop, proxy, rx);

    // Start event loop and block until shutdown.
    let result = processor.run(window_event_loop);

    info!("Goodbye");

    result
}
