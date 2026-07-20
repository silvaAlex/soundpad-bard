use crate::application::ports::ObsConnector;
use crate::domain::entities::AppConfig;
use anyhow::Result;

pub struct PauseBard<'a, O: ObsConnector> {
    pub obs: &'a O,
    pub config: &'a AppConfig,
}

impl<'a, O: ObsConnector> PauseBard<'a, O> {
    pub async fn execute(&self) -> Result<()> {
        self.obs
            .set_source_volume(&self.config.obs.bgm_source, 0.0)
            .await?;
        Ok(())
    }
}
