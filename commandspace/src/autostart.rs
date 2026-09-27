#[cfg(target_os = "macos")]
pub use crate::macos::autostart::{is_enabled, set_start_at_login};

#[cfg(all(unix, not(target_os = "macos")))]
mod unix_autostart {
    use std::fs;
    use std::path::PathBuf;

    const DESKTOP_TEMPLATE: &str = include_str!("../../extra/linux/CommandSpace.desktop");

    fn autostart_file_path() -> Option<PathBuf> {
        let config_dir = dirs::config_dir()?;
        Some(config_dir.join("autostart").join("commandspace.desktop"))
    }

    pub fn is_enabled() -> bool {
        autostart_file_path().is_some_and(|p| p.exists())
    }

    pub fn set_start_at_login(enable: bool) -> Result<(), String> {
        let path = autostart_file_path().ok_or_else(|| "Failed to locate user config directory".to_string())?;

        if enable {
            let exe = std::env::current_exe().map_err(|e| format!("Failed to get executable path: {e}"))?;
            let exe_str = format!("Exec=\"{}\"", exe.to_string_lossy());
            let content = DESKTOP_TEMPLATE
                .replace("Exec=commandspace", &exe_str)
                .replace("TryExec=commandspace\n", "");

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Failed to create autostart directory: {e}"))?;
            }
            fs::write(&path, content).map_err(|e| format!("Failed to write autostart desktop entry: {e}"))?;
        } else if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to remove autostart desktop entry: {e}"))?;
        }

        Ok(())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub use unix_autostart::{is_enabled, set_start_at_login};

#[cfg(windows)]
mod windows_autostart {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APP_NAME: &str = "CommandSpace";

    pub fn is_enabled() -> bool {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let Ok(key) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ) else {
            return false;
        };
        key.get_value::<String, _>(APP_NAME).is_ok()
    }

    pub fn set_start_at_login(enable: bool) -> Result<(), String> {
        let current_exe = std::env::current_exe().map_err(|e| format!("Failed to get executable path: {e}"))?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        if enable {
            let (key, _) = hkcu
                .create_subkey(RUN_KEY)
                .map_err(|e| format!("Failed to open registry Run key: {e}"))?;
            let formatted_path = format!("\"{}\"", current_exe.to_string_lossy());
            key.set_value(APP_NAME, &formatted_path)
                .map_err(|e| format!("Failed to set autostart registry value: {e}"))?;
        } else if let Ok(key) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) {
            let _ = key.delete_value(APP_NAME);
        }

        Ok(())
    }
}

#[cfg(windows)]
pub use windows_autostart::{is_enabled, set_start_at_login};

#[cfg(not(any(unix, windows)))]
pub fn is_enabled() -> bool {
    false
}

#[cfg(not(any(unix, windows)))]
pub fn set_start_at_login(_enable: bool) -> Result<(), String> {
    Ok(())
}
