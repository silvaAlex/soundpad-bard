use std::path::PathBuf;

/// Canal de áudio: Bgm é o Bard (loop de fundo), Sfx é o Soundpad (efeitos pontuais).
/// A separação em canal, não em "modo", é o que permite os dois coexistirem
/// e o ducking reagir a eventos de um canal sobre o outro.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioChannel {
    Bgm,
    Sfx,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioClip {
    pub id: String,
    pub path: PathBuf,
    pub channel: AudioChannel,
    /// Volume base configurado pelo usuário, 0.0 a 1.0.
    /// Para o Bgm, este é o volume "de repouso" (ex: 0.30 = lofi baixo).
    /// O volume efetivo durante ducking é calculado pela DuckingPolicy,
    /// não sobrescrito aqui — mantém o valor base sempre recuperável.
    pub base_volume: f32,
}

impl AudioClip {
    #[allow(dead_code)]
    pub fn new(id: impl Into<String>, path: PathBuf, channel: AudioChannel, base_volume: f32) -> Self {
        Self {
            id: id.into(),
            path,
            channel,
            base_volume: base_volume.clamp(0.0, 1.0),
        }
    }
}
