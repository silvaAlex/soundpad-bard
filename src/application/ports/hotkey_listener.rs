use crate::domain::entities::Hotkey;
use anyhow::Result;

pub trait HotkeyListener {
    fn register(&mut self, hotkey: &Hotkey) -> Result<u32>;
    fn unregister(&mut self, id: u32) -> Result<()>;
    fn poll_events(&mut self) -> Vec<u32>;
}
