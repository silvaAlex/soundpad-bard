use crate::application::ports::{ConfigRepository, ObsConnector};
use anyhow::Result;

#[allow(dead_code)]
pub struct PauseBard<'a, O: ObsConnector, C: ConfigRepository> {
    pub obs: &'a mut O,
    pub config: &'a C,
}

#[allow(dead_code)]
impl<'a, O: ObsConnector, C: ConfigRepository> PauseBard<'a, O, C> {
    pub async fn execute(&mut self) -> Result<()> {
        let config = self.config.load()?;
        self.obs
            .set_source_volume(&config.obs.bgm_source, 0.0)
            .await?;
        Ok(())
    }
}
