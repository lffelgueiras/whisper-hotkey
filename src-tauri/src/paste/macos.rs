use super::Paster;
use crate::error::AppError;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

const KEY_V: CGKeyCode = 9;

pub struct MacPaster;

impl MacPaster {
    pub fn new() -> Self { Self }
}

impl Paster for MacPaster {
    fn paste(&self, text: &str) -> Result<(), AppError> {
        let mut cb = arboard::Clipboard::new()
            .map_err(|e| AppError::Paste(format!("clipboard: {e}")))?;
        cb.set_text(text.to_string())
            .map_err(|e| AppError::Paste(format!("set clipboard: {e}")))?;
        let src = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .map_err(|_| AppError::Paste("CGEventSource".into()))?;
        let down = CGEvent::new_keyboard_event(src.clone(), KEY_V, true)
            .map_err(|_| AppError::Paste("key down".into()))?;
        down.set_flags(CGEventFlags::CGEventFlagCommand);
        let up = CGEvent::new_keyboard_event(src, KEY_V, false)
            .map_err(|_| AppError::Paste("key up".into()))?;
        up.set_flags(CGEventFlags::CGEventFlagCommand);
        down.post(CGEventTapLocation::HID);
        up.post(CGEventTapLocation::HID);
        Ok(())
    }
}
