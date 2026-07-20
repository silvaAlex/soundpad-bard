use crate::application::ports::HotkeyListener;
use crate::domain::entities::Hotkey;
use anyhow::{Context, Result};
use global_hotkey::{hotkey::HotKey, GlobalHotKeyEvent, GlobalHotKeyManager};
use std::collections::HashMap;
use std::str::FromStr;

pub struct GlobalHotkeyListener {
    manager: GlobalHotKeyManager,
    registered: HashMap<u32, HotKey>,
}

impl GlobalHotkeyListener {
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        Ok(Self {
            manager,
            registered: HashMap::new(),
        })
    }
}

impl Default for GlobalHotkeyListener {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl HotkeyListener for GlobalHotkeyListener {
    fn register(&mut self, hotkey: &Hotkey) -> Result<u32> {
        let hk = HotKey::from_str(hotkey.as_str())
            .with_context(|| format!("hotkey invalida para o SO: '{}'", hotkey.normalized()))?;
        self.manager
            .register(hk)
            .with_context(|| format!("falha ao registrar '{}' no SO", hotkey.normalized()))?;
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

    fn poll_events(&mut self) -> Vec<u32> {
        let mut fired = Vec::new();
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            fired.push(event.id);
        }
        fired
    }
}
