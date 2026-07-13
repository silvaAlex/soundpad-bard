use crate::application::ports::HotkeyListener;
use crate::domain::entities::Hotkey;
use anyhow::{Context, Result};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyManager};
use std::collections::HashMap;
use std::str::FromStr;

#[allow(dead_code)]
pub struct GlobalHotkeyListener {
    manager: GlobalHotKeyManager,
    registered: HashMap<u32, HotKey>,
}

#[allow(dead_code)]
impl GlobalHotkeyListener {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .context("falha ao iniciar o gerenciador de hotkeys do SO")?;
        Ok(Self {
            manager,
            registered: HashMap::new(),
        })
    }

    pub fn manager(&self) -> &GlobalHotKeyManager {
        &self.manager
    }
}

#[allow(dead_code)]
impl HotkeyListener for GlobalHotkeyListener {
    fn register(&mut self, hotkey: &Hotkey) -> Result<u32> {
        let hk = HotKey::from_str(hotkey.as_str())
            .with_context(|| format!("hotkey invalida para o SO: '{}'", hotkey.as_str()))?;
        self.manager
            .register(hk)
            .with_context(|| format!("falha ao registrar hotkey '{}' no SO", hotkey.as_str()))?;
        let id = hk.id();
        self.registered.insert(id, hk);
        Ok(id)
    }

    fn unregister(&mut self, id: u32) -> Result<()> {
        if let Some(hk) = self.registered.remove(&id) {
            self.manager
                .unregister(hk)
                .context("falha ao desregistrar hotkey")?;
        }
        Ok(())
    }
}
