use crate::domain::entities::Hotkey;
use anyhow::Result;

/// Porta para o listener de hotkeys globais do SO.
#[allow(dead_code)]
pub trait HotkeyListener {
    fn register(&mut self, hotkey: &Hotkey) -> Result<u32>;
    fn unregister(&mut self, id: u32) -> Result<()>;
}
