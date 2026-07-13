use crate::application::ports::ObsConnector;
use anyhow::{Context, Result};
use obws::Client;

pub struct ObwsConnector {
    host: String,
    port: u16,
    password: Option<String>,
    client: Option<Client>,
}

impl ObwsConnector {
    pub fn new(host: impl Into<String>, port: u16, password: Option<String>) -> Self {
        Self {
            host: host.into(),
            port,
            password,
            client: None,
        }
    }

    #[allow(dead_code)]
    fn client(&self) -> Result<&Client> {
        self.client
            .as_ref()
            .context("ObwsConnector nao conectado -- chame connect() primeiro")
    }
}

impl ObsConnector for ObwsConnector {
    async fn connect(&mut self) -> Result<()> {
        let client = Client::connect(&self.host, self.port, self.password.as_deref())
            .await
            .context("falha ao conectar no OBS via WebSocket")?;
        self.client = Some(client);
        Ok(())
    }

    async fn play_sfx(&self, source_name: &str, file_path: &str) -> Result<()> {
        let client = self.client()?;
        client
            .inputs()
            .set_settings(obws::requests::inputs::SetSettings {
                input: obws::requests::inputs::InputId::Name(source_name),
                settings: &serde_json::json!({ "local_file": file_path }),
                overlay: Some(true),
            })
            .await
            .context("falha ao trocar arquivo da media source")?;

        client
            .media_inputs()
            .trigger_action(
                obws::requests::inputs::InputId::Name(source_name),
                obws::common::MediaAction::Restart,
            )
            .await
            .context("falha ao dar play na media source")?;

        Ok(())
    }

    async fn set_source_volume(&self, source_name: &str, volume: f32) -> Result<()> {
        let client = self.client()?;
        client
            .inputs()
            .set_volume(
                obws::requests::inputs::InputId::Name(source_name),
                obws::requests::inputs::Volume::Mul(volume),
            )
            .await
            .context("falha ao ajustar volume da source")?;
        Ok(())
    }

    async fn set_ducking_filter_enabled(
        &self,
        source_name: &str,
        filter_name: &str,
        enabled: bool,
    ) -> Result<()> {
        let client = self.client()?;
        client
            .filters()
            .set_enabled(obws::requests::filters::SetEnabled {
                source: obws::requests::sources::SourceId::Name(source_name),
                filter: filter_name,
                enabled,
            })
            .await
            .context("falha ao alternar filtro de ducking")?;
        Ok(())
    }
}
