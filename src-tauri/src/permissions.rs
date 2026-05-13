use serde::Serialize;

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct PermissionStatus {
    pub accessibility: bool,
    pub microphone: bool,
}

#[cfg(target_os = "macos")]
pub fn check() -> PermissionStatus {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    let accessibility = unsafe { AXIsProcessTrusted() };
    PermissionStatus {
        accessibility,
        microphone: true,
    }
}

#[cfg(target_os = "windows")]
pub fn check() -> PermissionStatus {
    PermissionStatus {
        accessibility: true,
        microphone: true,
    }
}

#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .status()?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn open_accessibility_settings() -> std::io::Result<()> {
    Ok(())
}
