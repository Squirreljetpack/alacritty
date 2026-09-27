use objc2_service_management::{SMAppService, SMAppServiceStatus};

pub fn is_enabled() -> bool {
    unsafe {
        let service = SMAppService::mainAppService();
        service.status() == SMAppServiceStatus::Enabled
    }
}

pub fn set_start_at_login(enable: bool) -> Result<(), String> {
    // Clean up any legacy LaunchAgent plist from older auto-launch versions
    if let Some(home) = home::home_dir() {
        let legacy_plist = home.join("Library/LaunchAgents/CommandSpace.plist");
        if legacy_plist.exists() {
            let _ = std::fs::remove_file(legacy_plist);
        }
    }

    unsafe {
        let service = SMAppService::mainAppService();
        let current_status = service.status();

        if enable {
            if current_status != SMAppServiceStatus::Enabled {
                service
                    .registerAndReturnError()
                    .map_err(|e| format!("Failed to register login item via SMAppService: {e}"))?;
            }
        } else {
            if current_status == SMAppServiceStatus::Enabled {
                service
                    .unregisterAndReturnError()
                    .map_err(|e| format!("Failed to unregister login item via SMAppService: {e}"))?;
            }
        }
    }

    Ok(())
}
