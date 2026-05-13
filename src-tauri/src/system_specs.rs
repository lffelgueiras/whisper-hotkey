use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../src/ipc/generated/")]
pub struct SystemSpecs {
    /// Total installed RAM in gigabytes (1 GB = 1024³ bytes).
    pub ram_gb: f32,
}

pub fn detect() -> SystemSpecs {
    SystemSpecs {
        ram_gb: total_ram_bytes() as f32 / (1024.0 * 1024.0 * 1024.0),
    }
}

#[cfg(target_os = "macos")]
fn total_ram_bytes() -> u64 {
    use std::ffi::CString;
    let key = CString::new("hw.memsize").unwrap();
    let mut value: u64 = 0;
    let mut size = std::mem::size_of::<u64>();
    let rc = unsafe {
        libc::sysctlbyname(
            key.as_ptr(),
            &mut value as *mut _ as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if rc == 0 {
        value
    } else {
        0
    }
}

#[cfg(target_os = "windows")]
fn total_ram_bytes() -> u64 {
    use windows::Win32::System::SystemInformation::{
        GlobalMemoryStatusEx, MEMORYSTATUSEX,
    };
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe {
        if GlobalMemoryStatusEx(&mut status).is_ok() {
            status.ullTotalPhys
        } else {
            0
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn total_ram_bytes() -> u64 {
    0
}
