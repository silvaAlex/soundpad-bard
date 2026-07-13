use crate::application::ports::{ConfigRepository, ObsConnector};
use crate::domain::entities::SoundpadClip;
use anyhow::{Context, Result};

#[allow(dead_code)]
pub struct PlaySoundEffect<'a, O: ObsConnector, C: ConfigRepository> {
    pub obs: &'a mut O,
    pub config: &'a C,
}

#[allow(dead_code)]
impl<'a, O: ObsConnector, C: ConfigRepository> PlaySoundEffect<'a, O, C> {
    pub async fn execute(&mut self, clip_id: &str) -> Result<()> {
        let config = self.config.load()?;
        let clip: &SoundpadClip = config
            .soundpad
            .clips
            .iter()
            .find(|c| c.id == clip_id)
            .context("clip not found")?;

        let file_path = config.soundpad.sfx_dir.join(&clip.filename);
        let file_str = file_path.to_str().context("caminho invalido")?;

        self.obs
            .set_source_volume(&config.obs.sfx_source, clip.volume)
            .await?;
        self.obs
            .play_sfx(&config.obs.sfx_source, file_str)
            .await?;

        Ok(())
    }
}
