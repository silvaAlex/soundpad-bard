use crate::domain::entities::Hotkey;
use anyhow::Result;

//application/ports/hotkey_listener

/// Porta para o listener de hotkeys globais do SO.
pub trait HotkeyListener {
    fn register(&mut self, hotkey: &Hotkey) -> Result<u32>;
    fn unregister(&mut self, id: u32) -> Result<()>;

    /// Drena todos os eventos de hotkey pendentes e retorna os IDs disparados.
    fn poll_events(&mut self) -> Vec<u32>;
}
