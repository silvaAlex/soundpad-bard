use crate::application::ports::ConfigRepository;
use crate::domain::entities::AppConfig;
use crate::infrastructure::crypto::{decrypt_password, encrypt_password};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct JsonConfigRepository {
    path: PathBuf,
}

impl JsonConfigRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/config.json")
    }
}

impl ConfigRepository for JsonConfigRepository {
    fn load(&self) -> Result<AppConfig> {
        if !self.path.exists() {
            let config = AppConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        let content =
            std::fs::read_to_string(&self.path).context("falha ao ler arquivo de configuracao")?;
        let mut config: AppConfig =
            serde_json::from_str(&content).context("falha ao parsear arquivo de configuracao")?;

        if let Some(ref pwd) = config.obs.password {
            config.obs.password = Some(
                decrypt_password(pwd).context("falha ao descriptografar senha do OBS")?,
            );
        }

        Ok(config)
    }

    fn save(&self, config: &AppConfig) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).context("falha ao criar diretorio de configuracao")?;
        }

        let mut config_to_save = config.clone();
        if let Some(ref pwd) = config_to_save.obs.password {
            config_to_save.obs.password = Some(
                encrypt_password(pwd).context("falha ao criptografar senha do OBS")?,
            );
        }

        let content = serde_json::to_string_pretty(&config_to_save)
            .context("falha ao serializar configuracao")?;

        let tmp_path = self.path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &content)
            .context("falha ao escrever arquivo de configuracao temporario")?;
        std::fs::rename(&tmp_path, &self.path)
            .context("falha ao renomear arquivo de configuracao")?;

        Ok(())
    }
}
