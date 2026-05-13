use crate::error::AppError;

pub trait Paster: Send + Sync {
    fn paste(&self, text: &str) -> Result<(), AppError>;
}

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub fn default_paster() -> Box<dyn Paster> {
    Box::new(macos::MacPaster::new())
}

#[cfg(target_os = "windows")]
pub fn default_paster() -> Box<dyn Paster> {
    Box::new(windows::WinPaster::new())
}
