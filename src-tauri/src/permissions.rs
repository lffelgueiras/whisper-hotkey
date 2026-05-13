use serde::Serialize;

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct PermissionStatus {
    pub accessibility: bool,
    pub microphone: bool,
}

#[cfg(target_os = "macos")]
pub fn check() -> PermissionStatus {
    check_with_prompt(false)
}

#[cfg(target_os = "macos")]
pub fn check_with_prompt(prompt: bool) -> PermissionStatus {
    use core_foundation::base::TCFType;
    use core_foundation::boolean::CFBoolean;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::{CFString, CFStringRef};

    extern "C" {
        fn AXIsProcessTrustedWithOptions(options: *const std::ffi::c_void) -> bool;
        static kAXTrustedCheckOptionPrompt: CFStringRef;
    }

    let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let value = CFBoolean::from(prompt);
    let dict = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
    let accessibility =
        unsafe { AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as *const _) };
    PermissionStatus {
        accessibility,
        microphone: microphone_authorized(),
    }
}

#[cfg(target_os = "macos")]
#[link(name = "AVFoundation", kind = "framework")]
extern "C" {}

#[cfg(target_os = "macos")]
fn microphone_status() -> i64 {
    use objc::runtime::Class;
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let cls: &Class = class!(AVCaptureDevice);
        let media: *mut objc::runtime::Object =
            msg_send![class!(NSString), stringWithUTF8String: b"soun\0".as_ptr()];
        msg_send![cls, authorizationStatusForMediaType: media]
    }
}

#[cfg(target_os = "macos")]
fn microphone_authorized() -> bool {
    microphone_status() == 3
}

#[cfg(target_os = "macos")]
pub fn request_microphone_if_needed() {
    let status = microphone_status();
    if status != 0 {
        return;
    }
    use objc::runtime::Class;
    use objc::{class, msg_send, sel, sel_impl};
    use block::ConcreteBlock;
    unsafe {
        let cls: &Class = class!(AVCaptureDevice);
        let media: *mut objc::runtime::Object =
            msg_send![class!(NSString), stringWithUTF8String: b"soun\0".as_ptr()];
        let block = ConcreteBlock::new(|_granted: bool| {});
        let block = block.copy();
        let _: () = msg_send![cls, requestAccessForMediaType: media completionHandler: &*block];
    }
}

#[cfg(target_os = "macos")]
pub fn ensure_accessibility_prompt() {
    if !check_with_prompt(false).accessibility {
        let _ = check_with_prompt(true);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_microphone() {}

#[cfg(not(target_os = "macos"))]
pub fn check_with_prompt(_prompt: bool) -> PermissionStatus {
    check()
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
