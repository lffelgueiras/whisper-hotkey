use crate::error::AppError;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager};
use std::str::FromStr;
use tokio::sync::mpsc::UnboundedSender;

pub struct HotkeyService {
    manager: GlobalHotKeyManager,
    current: Option<HotKey>,
}

impl HotkeyService {
    pub fn new() -> Result<Self, AppError> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| AppError::Hotkey(format!("manager init: {e}")))?;
        Ok(Self {
            manager,
            current: None,
        })
    }

    pub fn register(&mut self, accelerator: &str) -> Result<(), AppError> {
        let hk = HotKey::from_str(accelerator)
            .map_err(|e| AppError::Hotkey(format!("parse '{accelerator}': {e}")))?;
        if let Some(prev) = self.current.take() {
            let _ = self.manager.unregister(prev);
        }
        self.manager
            .register(hk)
            .map_err(|e| AppError::Hotkey(format!("register: {e}")))?;
        self.current = Some(hk);
        Ok(())
    }

    pub fn start_listener(tx: UnboundedSender<()>) {
        std::thread::spawn(move || {
            let rx = GlobalHotKeyEvent::receiver();
            loop {
                match rx.recv() {
                    Ok(_event) => {
                        let _ = tx.send(());
                    }
                    Err(e) => {
                        tracing::error!("hotkey rx closed: {e}");
                        break;
                    }
                }
            }
        });
    }
}
