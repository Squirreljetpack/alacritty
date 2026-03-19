use cba::bo::{load_type, load_type_or_default_log};
use cba::unwrap;
use commandspace_config::LOG_TARGET_CONFIG;
use commandspace_config::action::WindowAction;
use commandspace_config::global_bindings::{GlobalAction, GlobalBindings};
use commandspace_config::paths::cb_config_path;
use mm_clipboard_config::ServerConfig;

use crate::cli::Options;

use crate::config::AlacrittyConfig;
use crate::config::configs::{AlacrittyConfigGeneral, AlacrittyConfigSpecific, Config};
use crate::config::terminal::Terminal;
use crate::config::types::Program;
use crate::paths::{alacritty_config_path, config_path};

/// Load the configuration file.
pub fn load(options: &mut Options) -> (AlacrittyConfig, Config, ServerConfig) {
    let specific_cfg: AlacrittyConfigSpecific =
        load_type_or_default_log(alacritty_config_path(), |s| toml::from_str(s));
    let cfg: Config = load_type_or_default_log(config_path(), |s| toml::from_str(s));

    let cb_cfg: mm_clipboard_config::Config =
        load_type_or_default_log(cb_config_path(), |s| toml::from_str(s));

    // cfg.alacritty remains in the main cfg
    let mut alacritty_cfg = specific_into_alacritty_config(specific_cfg, cfg.alacritty.clone());

    // Override config with CLI options.
    options.override_config(&mut alacritty_cfg);

    (alacritty_cfg, cfg, cb_cfg.server)
}

pub fn try_load_ui_config(options: &Options) -> Option<(AlacrittyConfig, GlobalBindings)> {
    let specific_cfg: AlacrittyConfigSpecific = unwrap!(
        load_type(alacritty_config_path(), |s| toml::from_str(s));
        |e| {
            log::error!(target: LOG_TARGET_CONFIG, "Unable to load config {:?}: {e}", alacritty_config_path());
            None
        }
    );
    let mut cfg: Config = unwrap!(
        load_type(config_path(), |s| toml::from_str(s));
        |e| {
            log::error!(target: LOG_TARGET_CONFIG, "Unable to load config {:?}: {e}", config_path());
            None
        }
    );

    let mut alacritty_cfg = specific_into_alacritty_config(specific_cfg, cfg.alacritty);

    // Override config with CLI options.
    options.override_config(&mut alacritty_cfg);

    let key = cfg.bindings.1;
    let action = GlobalAction::Window(WindowAction::Toggle);

    if let Some((_, a)) = cfg.bindings.0.iter_mut().find(|(k, _)| *k == key) {
        *a = action;
    } else {
        cfg.bindings.0.push((key, action));
    }

    Some((alacritty_cfg, cfg.bindings))
}

pub fn specific_into_alacritty_config(
    specific: AlacrittyConfigSpecific,
    cfg: AlacrittyConfigGeneral,
) -> AlacrittyConfig {
    let AlacrittyConfigSpecific { mouse, bell, hints, keyboard } = specific;
    let AlacrittyConfigGeneral {
        env,

        window,
        colors,
        font,

        cursor,
        scrolling,
        selection,

        debug,
        default_command,
    } = cfg;
    let mut terminal = Terminal::default();
    terminal.shell = Some(default_command.unwrap_or_else(default_program));

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

// pub fn save(config: &Config, alacritty_cfg: &AlacrittyConfig) {
//     let specific_cfg: AlacrittyConfigSpecific = specific_from_alacritty_config(alacritty_cfg);
//     dump_type(config_path(), config, toml::to_string)._elog();
//     dump_type(alacritty_config_path(), &specific_cfg, toml::to_string)._elog();
// }

// pub fn specific_from_alacritty_config(cfg: &AlacrittyConfig) -> AlacrittyConfigSpecific {
//     let AlacrittyConfig { env, scrolling, selection, font, mouse, bell, hints, keyboard, .. } =
//         cfg.clone();

//     AlacrittyConfigSpecific { env, scrolling, selection, font, mouse, bell, hints, keyboard }
// }

// todo: lowpri: make this configurable
use crate::paths::fzs_path;
fn default_program() -> Program {
    Program::WithArgs { program: fzs_path().to_path_buf(), args: vec!["run".into(), "main".into()] }
}
