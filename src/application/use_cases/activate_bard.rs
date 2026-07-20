use crate::application::ports::ObsConnector;
use crate::domain::entities::AppConfig;
use anyhow::{Context, Result};

pub struct ActivateBard<'a, O: ObsConnector> {
    pub obs: &'a O,
    pub config: &'a AppConfig,
}

impl<'a, O: ObsConnector> ActivateBard<'a, O> {
    pub async fn execute(&self, music_file: &str) -> Result<()> {
        let file_path = self.config.bard.music_dir.join(music_file);
        let file_str = file_path.to_str().context("caminho invalido")?;

        self.obs
            .set_source_volume(&self.config.obs.bgm_source, self.config.bard.volume)
            .await?;
        self.obs
            .play_sfx(&self.config.obs.bgm_source, file_str)
            .await?;

        Ok(())
    }
}
