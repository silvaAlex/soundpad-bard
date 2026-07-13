use crate::application::ports::ConfigRepository;
use crate::domain::entities::SoundpadClip;
use crate::domain::services::HotkeyConflictChecker;
use anyhow::{Context, Result};

#[allow(dead_code)]
pub struct RegisterHotkey<'a, C: ConfigRepository> {
    pub config: &'a C,
    pub checker: &'a mut HotkeyConflictChecker,
}

#[allow(dead_code)]
impl<'a, C: ConfigRepository> RegisterHotkey<'a, C> {
    pub fn execute(&mut self, clip_id: &str, hotkey_str: &str) -> Result<()> {
        let config = self.config.load()?;
        let _clip: &SoundpadClip = config
            .soundpad
            .clips
            .iter()
            .find(|c| c.id == clip_id)
            .context("clip not found")?;

        let hotkey = crate::domain::entities::Hotkey::parse(hotkey_str)
            .map_err(|e| anyhow::anyhow!(e))?;
        self.checker
            .register(hotkey)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
    }
}
