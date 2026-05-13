use super::Paster;
use crate::error::AppError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};

pub struct WinPaster;

impl WinPaster {
    pub fn new() -> Self {
        Self
    }
}

fn key_event(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

impl Paster for WinPaster {
    fn paste(&self, text: &str) -> Result<(), AppError> {
        let mut cb =
            arboard::Clipboard::new().map_err(|e| AppError::Paste(format!("clipboard: {e}")))?;
        cb.set_text(text.to_string())
            .map_err(|e| AppError::Paste(format!("set clipboard: {e}")))?;
        let inputs = [
            key_event(VK_CONTROL, false),
            key_event(VK_V, false),
            key_event(VK_V, true),
            key_event(VK_CONTROL, true),
        ];
        let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
        if sent as usize != inputs.len() {
            return Err(AppError::Paste(format!(
                "SendInput sent {sent}/{}",
                inputs.len()
            )));
        }
        Ok(())
    }
}
