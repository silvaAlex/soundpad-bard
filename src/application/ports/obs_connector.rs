use anyhow::Result;

pub trait ObsConnector: Send {
    async fn connect(&mut self) -> Result<()>;

    async fn play_sfx(&self, source_name: &str, file_path: &str) -> Result<()>;
    async fn set_source_volume(&self, source_name: &str, volume: f32) -> Result<()>;

    async fn set_ducking_filter_enabled(
        &self,
        source_name: &str,
        filter_name: &str,
        enabled: bool,
    ) -> Result<()>;
}
