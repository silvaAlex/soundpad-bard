use crate::application::ports::ObsConnector;
use crate::domain::entities::{AppConfig, AudioClip};
use crate::domain::services::DuckingPolicy;
use anyhow::{Context, Result};

pub struct PlaySoundEffect<'a, O: ObsConnector> {
    pub obs: &'a O,
    pub config: &'a AppConfig,
}

impl<'a, O: ObsConnector> PlaySoundEffect<'a, O> {
    pub async fn execute(&self, clip_id: &str) -> Result<()> {
        let clip = self
            .config
            .soundpad
            .clips
            .iter()
            .find(|c| c.id == clip_id)
            .context("clip nao encontrado")?;

        let audio: AudioClip = clip.to_audio_clip(&self.config.soundpad.sfx_dir);
        let file_str = audio.path_str().context("caminho invalido")?;

        self.obs
            .set_source_volume(&self.config.obs.sfx_source, audio.base_volume)
            .await?;
        self.obs
            .play_sfx(&self.config.obs.sfx_source, file_str)
            .await?;

        let dc = &self.config.bard.ducking;
        let policy = DuckingPolicy {
            duck_ratio: dc.duck_ratio,
            attack_ms: dc.attack_ms,
            release_ms: dc.release_ms,
        };
        let bgm_base = self.config.bard.volume;
        let ducked_volume = policy.target_volume(bgm_base, 1);

        tokio::time::sleep(std::time::Duration::from_millis(policy.attack_ms as u64)).await;

        self.obs
            .set_source_volume(&self.config.obs.bgm_source, ducked_volume)
            .await?;
        self.obs
            .set_ducking_filter_enabled(
                &self.config.obs.bgm_source,
                &self.config.obs.ducking_filter,
                true,
            )
            .await?;

        tokio::time::sleep(std::time::Duration::from_millis(policy.release_ms as u64)).await;

        self.obs
            .set_source_volume(&self.config.obs.bgm_source, bgm_base)
            .await?;
        self.obs
            .set_ducking_filter_enabled(
                &self.config.obs.bgm_source,
                &self.config.obs.ducking_filter,
                false,
            )
            .await?;

        Ok(())
    }
}
