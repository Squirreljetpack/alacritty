#![allow(unused)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use alacritty_terminal::term::Config as TermConfig;
use alacritty_terminal::tty::Options as PtyOptions;

use super::types::Program;

use super::global_bindings::GlobalBindings;

use super::hint::Hints;
use super::types::Keyboard;

use super::bell::BellConfig;
use super::bg::BgConfig;
use super::bindings::{KeyBinding, MouseBinding};
use super::color::Colors;
use super::cursor::Cursor;
use super::debug::Debug;
use super::font::Font;
use super::mouse::Mouse;
use super::scrolling::Scrolling;
use super::selection::Selection;
use super::terminal::Terminal;
use super::window::WindowConfig;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct AlacrittyConfig {
    /// Extra environment variables.
    pub env: HashMap<String, String>,

    /// How much scrolling history to keep.
    pub scrolling: Scrolling,

    /// Cursor configuration.
    pub cursor: Cursor,

    /// Selection configuration.
    pub selection: Selection,

    /// Font configuration.
    pub font: Font,

    /// Window configuration.
    pub window: WindowConfig,

    /// Mouse configuration.
    pub mouse: Mouse,

    /// Debug options.
    pub debug: Debug,

    /// Bell configuration.
    pub bell: BellConfig,

    /// RGB values for colors.
    pub colors: Colors,

    /// Regex hints for interacting with terminal content.
    pub hints: Hints,

    /// Config for the alacritty_terminal itself.
    pub terminal: Terminal,

    /// Keyboard configuration.
    pub keyboard: Keyboard,
}

impl AlacrittyConfig {
    /// Derive [`TermConfig`] from the config.
    pub fn term_options(&self) -> TermConfig {
        TermConfig {
            semantic_escape_chars: self.selection.semantic_escape_chars.clone(),
            scrolling_history: self.scrolling.history() as usize,
            vi_mode_cursor_style: self.cursor.vi_mode_style(),
            default_cursor_style: self.cursor.style(),
            osc52: self.terminal.osc52.0,
            kitty_keyboard: true,
        }
    }

    /// Derive [`PtyOptions`] from the config.
    pub fn pty_config(&self) -> PtyOptions {
        let shell = self.terminal.shell.clone().map(Into::into);
        // todo: understand then fold home_dir default into WorkingDir wrapper struct
        let working_directory =
            self.terminal.working_directory.clone().or_else(|| std::env::home_dir());
        PtyOptions {
            working_directory,
            shell,
            drain_on_exit: false,
            env: HashMap::new(),
            #[cfg(target_os = "windows")]
            escape_args: false,
        }
    }

    /// Derive [`BgOptions`] from the config.
    pub fn bg_config(&self) -> BgConfig {
        BgConfig {
            radius: self.window_radius(),
            bg_color: self.colors.primary.background,
            bg_alpha: self.window_opacity(),
            frame_color: Default::default(),
            frame_alpha: 0.0,
            frame_offset: 0.0,
            frame_thickness: 0.0,
        }
    }

    #[inline]
    pub fn window_opacity(&self) -> f32 {
        self.window.opacity.as_f32()
    }

    #[inline]
    pub fn window_radius(&self) -> f32 {
        self.window.radius as f32 / 50.0
    }

    #[inline]
    pub fn key_bindings(&self) -> &[KeyBinding] {
        &self.keyboard.bindings.0
    }

    #[inline]
    pub fn mouse_bindings(&self) -> &[MouseBinding] {
        &self.mouse.bindings.0
    }
}



/// The object deserialized from the main commandspace config file
#[derive(Default, Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)]
    pub alacritty: AlacrittyConfigGeneral,
    pub bindings: GlobalBindings,
    pub download: Download,
    pub stats: Stats,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Download {}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Stats {
    pub count: u8,
}

/// The alacritty object deserialized from the main commandspace config file
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct AlacrittyConfigGeneral {
    pub cursor: Cursor,
    pub window: WindowConfig,
    pub colors: Colors,
    pub debug: Debug,
    pub default_command: Option<Program>,
}

impl Default for AlacrittyConfigGeneral {
    fn default() -> Self {
        Self {
            cursor: Default::default(),
            window: Default::default(),
            colors: Default::default(),
            debug: Default::default(),
            default_command: None,
        }
    }
}

/// Alacritty settings which should be set manually in the alacritty.toml
#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[serde(deny_unknown_fields, default)]
pub struct AlacrittyConfigSpecific {
    pub env: HashMap<String, String>,

    pub scrolling: Scrolling,

    pub selection: Selection,
    pub font: Font,
    pub mouse: Mouse,
    pub bell: BellConfig,

    pub hints: Hints,
    pub keyboard: Keyboard,
}
