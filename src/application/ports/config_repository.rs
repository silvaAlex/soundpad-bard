use crate::domain::entities::AppConfig;
use anyhow::Result;

pub trait ConfigRepository {
    fn load(&self) -> Result<AppConfig>;
    fn save(&self, config: &AppConfig) -> Result<()>;
}
