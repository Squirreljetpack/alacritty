pub mod config;
pub mod parse;

use std::cmp::max;
use std::collections::HashMap;
use std::path::PathBuf;

use clap::{ArgAction, Parser};
use log::{LevelFilter, error};
use serde::{Deserialize, Serialize};

use alacritty_terminal::tty::Options as PtyOptions;

use crate::config::AlacrittyConfig;
use crate::config::types::Program;

/// CLI options for the main Alacritty executable.
#[derive(Parser, Default, Debug)]
#[clap(author, about, version = env!("VERSION"))]
pub struct Options {
    /// Print all events to STDOUT.
    #[clap(long)]
    pub print_events: bool,

    /// Reduces the level of verbosity (the min level is -qq).
    #[clap(short, conflicts_with("verbose"), action = ArgAction::Count)]
    quiet: u8,

    /// Increases the level of verbosity (the max level is -vvv).
    #[clap(short, conflicts_with("quiet"), action = ArgAction::Count)]
    verbose: u8,
}

impl Options {
    /// Override configuration file with options from the CLI.
    pub fn override_config(&mut self, config: &mut AlacrittyConfig) {
        config.debug.print_events |= self.print_events;
        config.debug.log_level = max(config.debug.log_level, self.log_level());

        if config.debug.print_events {
            config.debug.log_level = max(config.debug.log_level, LevelFilter::Info);
        }
    }

    /// Logging filter level.
    pub fn log_level(&self) -> LevelFilter {
        match (self.quiet, self.verbose) {
            // Force at least `Info` level for `--print-events`.
            (_, 0) if self.print_events => LevelFilter::Info,

            // Default.
            (0, 0) => LevelFilter::Warn,

            // Verbose.
            (_, 1) => LevelFilter::Info,
            (_, 2) => LevelFilter::Debug,
            (0, _) => LevelFilter::Trace,

            // Quiet.
            (1, _) => LevelFilter::Error,
            (..) => LevelFilter::Off,
        }
    }

    /// Logging filter level.
    pub fn bog_level(&self) -> u8 {
        if self.verbose > 0 { 2 + self.verbose } else { 2u8.saturating_sub(self.quiet) }
    }
}

// This is used to create new windows, and was parsed from cli (flat inside IPC and Options).
// Since we eliminate cli overrides, parser methods are no longer required
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct WindowOptions {
    /// Terminal options which can be passed via IPC.
    pub terminal_options: TerminalOptions,

    #[cfg(not(any(target_os = "macos", windows)))]
    /// `ActivationToken` that we pass to winit.
    pub activation_token: Option<String>,

    /// Override configuration file options [example: 'cursor.style="Beam"'].
    option: Vec<String>,
}

// Since we eliminate cli overrides, we should parse a ptyoptions directly from a bound action. This should be moved to config.
/// Terminal specific cli options which can be passed to new windows via IPC.
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct TerminalOptions {
    /// Start the shell in the specified working directory.
    pub working_directory: Option<PathBuf>,

    /// Remain open after child process exit.
    pub hold: bool,

    /// Command and args to execute (must be last argument).
    command: Vec<String>,
}

impl TerminalOptions {
    /// Shell override passed through the CLI.
    pub fn command(&self) -> Option<Program> {
        let (program, args) = self.command.split_first()?;
        Some(Program::WithArgs { program: PathBuf::from(program.clone()), args: args.to_vec() })
    }

    /// Override the [`PtyOptions`]'s fields with the [`TerminalOptions`].
    pub fn override_pty_config(&self, pty_config: &mut PtyOptions) {
        if let Some(working_directory) = &self.working_directory {
            if working_directory.is_dir() {
                pty_config.working_directory = Some(working_directory.to_owned());
            } else {
                error!("Invalid working directory: {working_directory:?}");
            }
        }

        if let Some(command) = self.command() {
            pty_config.shell = Some(command.into());
        }

        pty_config.drain_on_exit |= self.hold;
    }
}

impl From<TerminalOptions> for PtyOptions {
    fn from(mut options: TerminalOptions) -> Self {
        PtyOptions {
            working_directory: options.working_directory.take(),
            shell: options.command().map(Into::into),
            drain_on_exit: options.hold,
            env: HashMap::new(),
            #[cfg(target_os = "windows")]
            escape_args: false,
        }
    }
}
