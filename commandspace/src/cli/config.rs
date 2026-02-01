#![allow(unused)]

use cli_boilerplate_automation::StringError;
use cli_boilerplate_automation::bait::ResultExt;
use cli_boilerplate_automation::bo::{dump_type, load_type, load_type_or_default};

use crate::cli::Options;

use crate::config::AlacrittyConfig;
use crate::config::configs::{AlacrittyConfigGeneral, AlacrittyConfigSpecific, Config};
use crate::config::terminal::Terminal;
use crate::config::types::Program;
use crate::paths::{alacritty_config_path, config_path};

/// Load the configuration file.
pub fn load(options: &mut Options) -> (AlacrittyConfig, Config) {
    let specific_cfg: AlacrittyConfigSpecific =
        load_type_or_default(alacritty_config_path(), |s| toml::from_str(s));
    let cfg: Config = load_type_or_default(config_path(), |s| toml::from_str(s));

    let mut alacritty_cfg = specific_into_alacritty_config(specific_cfg, &cfg.alacritty);

    // Override config with CLI options.
    options.override_config(&mut alacritty_cfg);

    (alacritty_cfg, cfg)
}

pub fn try_load_ui_config(options: &mut Options) -> Result<AlacrittyConfig, StringError> {
    let specific_cfg: AlacrittyConfigSpecific =
        load_type(alacritty_config_path(), |s| toml::from_str(s))?;
    let cfg: Config = load_type(config_path(), |s| toml::from_str(s))?;
    let mut alacritty_cfg = specific_into_alacritty_config(specific_cfg, &cfg.alacritty);

    // Override config with CLI options.
    options.override_config(&mut alacritty_cfg);

    Ok(alacritty_cfg)
}

pub fn save(config: &Config, alacritty_cfg: &AlacrittyConfig) {
    let specific_cfg: AlacrittyConfigSpecific = specific_from_alacritty_config(alacritty_cfg);
    dump_type(config_path(), config, toml::to_string)._elog();
    dump_type(alacritty_config_path(), &specific_cfg, toml::to_string)._elog();
}

pub fn specific_into_alacritty_config(
    specific: AlacrittyConfigSpecific,
    cfg: &AlacrittyConfigGeneral,
) -> AlacrittyConfig {
    let AlacrittyConfigSpecific { env, scrolling, selection, font, mouse, bell, hints, keyboard } =
        specific;
    let AlacrittyConfigGeneral { cursor, window, colors, debug, .. } = cfg.clone();
    let mut terminal = Terminal::default();
    terminal.shell =
        if let Some(p) = cfg.default_command.clone() { Some(p) } else { Some(default_program()) };

    AlacrittyConfig {
        env,
        scrolling,
        cursor,
        selection,
        font,
        window,
        mouse,
        debug,
        bell,
        colors,
        hints,
        terminal,
        keyboard,
    }
}

pub fn specific_from_alacritty_config(cfg: &AlacrittyConfig) -> AlacrittyConfigSpecific {
    let AlacrittyConfig { env, scrolling, selection, font, mouse, bell, hints, keyboard, .. } =
        cfg.clone();

    AlacrittyConfigSpecific { env, scrolling, selection, font, mouse, bell, hints, keyboard }
}

fn default_program() -> Program {
    Program::WithArgs {
        program: fzs_path().to_path_buf(),
        args: vec!["run".into(), "launch".into()],
    }
}
use crate::paths::fzs_path;
