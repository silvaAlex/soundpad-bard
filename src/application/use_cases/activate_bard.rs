use crate::application::ports::{ConfigRepository, ObsConnector};
use anyhow::{Context, Result};

#[allow(dead_code)]
pub struct ActivateBard<'a, O: ObsConnector, C: ConfigRepository> {
    pub obs: &'a mut O,
    pub config: &'a C,
}

#[allow(dead_code)]
impl<'a, O: ObsConnector, C: ConfigRepository> ActivateBard<'a, O, C> {
    pub async fn execute(&mut self, music_file: &str) -> Result<()> {
        let config = self.config.load()?;
        let file_path = config.bard.music_dir.join(music_file);
        let file_str = file_path.to_str().context("caminho invalido")?;

        self.obs
            .play_sfx(&config.obs.bgm_source, file_str)
            .await?;

        Ok(())
    }
}
