use anyhow::Result;

/// Porta para o OBS. A camada application só depende disso — não sabe
/// se por baixo é `obws`/WebSocket ou, futuramente, bindings de plugin nativo.
pub trait ObsConnector: Send {
    async fn connect(&mut self) -> Result<()>;

    /// Toca um SFX pontual numa media source do OBS.
    async fn play_sfx(&self, source_name: &str, file_path: &str) -> Result<()>;

    /// Ajusta o volume de uma source (usado pelo Bard e para ducking manual,
    /// caso o filtro de compressor nativo não seja usado).
    async fn set_source_volume(&self, source_name: &str, volume: f32) -> Result<()>;

    /// Liga/desliga o filtro de ducking (ex: Compressor com sidechain)
    /// aplicado na source do Bard.
    async fn set_ducking_filter_enabled(&self, source_name: &str, filter_name: &str, enabled: bool) -> Result<()>;
}
