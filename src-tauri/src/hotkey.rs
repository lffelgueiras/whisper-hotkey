use crate::error::AppError;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::str::FromStr;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Copy)]
pub enum HotkeyEdge {
    Press,
    Release,
}

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
        tracing::info!("hotkey register: '{}'", accelerator);
        if let Some(prev) = self.current.take() {
            let _ = self.manager.unregister(prev);
        }
        let hk = HotKey::from_str(accelerator)
            .map_err(|e| AppError::Hotkey(format!("parse '{accelerator}': {e}")))?;
        self.manager
            .register(hk)
            .map_err(|e| AppError::Hotkey(format!("register: {e}")))?;
        self.current = Some(hk);
        Ok(())
    }

    pub fn unregister(&mut self) {
        if let Some(prev) = self.current.take() {
            let _ = self.manager.unregister(prev);
        }
    }

    pub fn start_listener(tx: UnboundedSender<HotkeyEdge>) {
        std::thread::spawn(move || {
            let rx = GlobalHotKeyEvent::receiver();
            loop {
                match rx.recv() {
                    Ok(event) => {
                        tracing::info!("hotkey event: {:?}", event.state);
                        let edge = match event.state {
                            HotKeyState::Pressed => HotkeyEdge::Press,
                            HotKeyState::Released => HotkeyEdge::Release,
                        };
                        let _ = tx.send(edge);
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

