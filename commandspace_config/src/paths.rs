use cba::{bath::root_dir, bog::BogUnwrapExt, expr_as_path_fn};
use std::{ffi::OsString, path::PathBuf};

pub static BINARY_FULL: &str = "commandspace";

// ---------------------- DIRS ----------------------
// config defaults
pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        ._ebog("Failed to determine cache directory") // exit if failed to determine
        .join(BINARY_FULL)
}

pub fn state_dir() -> PathBuf {
    if let Some(ret) = dirs::state_dir() {
        ret.join(BINARY_FULL)
    } else {
        dirs::home_dir()
            ._ebog("Failed to determine state directory")
            .join(".local")
            .join("state")
            .join(BINARY_FULL)
    }
}
// --------------------------------
pub fn config_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        let config = home.join(".config").join(BINARY_FULL);
        if config.exists() {
            return config;
        }
    };

    dirs::config_dir()._ebog("Failed to determine config directory").join(BINARY_FULL)
}
pub fn settings_command() -> PathBuf {
    #[cfg(debug_assertions)]
    return __home().join("gh/_fzs/commandspace-settings/src-tauri/target/debug/app");
    #[cfg(not(debug_assertions))]
    return __home().join("gh/_fzs/commandspace-settings/src-tauri/target/release/app");
}

pub fn current_exe() -> std::ffi::OsString {
    std::env::current_exe().map(OsString::from).unwrap_or(BINARY_FULL.into())
}

// the absolute home directory, or root
expr_as_path_fn!(__home, dirs::home_dir().unwrap_or(root_dir()));

// ---------------------- FILES ----------------------

expr_as_path_fn!(cb_config_path, config_dir().join("cb.toml"));

#[cfg(debug_assertions)]
expr_as_path_fn!(config_path, config_dir().join("dev.toml"));
#[cfg(not(debug_assertions))]
expr_as_path_fn!(config_path, config_dir().join("config.toml"));

#[cfg(debug_assertions)]
expr_as_path_fn!(alacritty_config_path, config_dir().join("alacritty.dev.toml"));
#[cfg(not(debug_assertions))]
expr_as_path_fn!(alacritty_config_path, config_dir().join("alacritty.toml"));

pub fn maybe_fzs_path() -> Option<PathBuf> {
    // return a cached (static) result
    // in app, the path should be fixed.
    // non_app usage is not a priority so we just use which.
    which::which("fzs").map_err(|e| eprintln!("{e}")).ok()
}

expr_as_path_fn!(
    fzs_path,
    maybe_fzs_path().unwrap_or(PathBuf::from("/home/archr/local/paths/hacks/fzs"))
);
