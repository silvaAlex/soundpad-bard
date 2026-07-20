use std::path::PathBuf;

/// Canal de áudio: Bgm é o Bard (loop de fundo), Sfx é o Soundpad (efeitos pontuais).
/// A separação em canal, não em "modo", é o que permite os dois coexistirem
/// e o ducking reagir a eventos de um canal sobre o outro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AudioChannel {
    Bgm,
    Sfx,
}

impl std::fmt::Display for AudioChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioChannel::Bgm => write!(f, "BGM"),
            AudioChannel::Sfx => write!(f, "SFX"),
        }
    }
}

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
    pub fn new(
        id: impl Into<String>,
        path: PathBuf,
        channel: AudioChannel,
        base_volume: f32,
    ) -> Self {
        Self {
            id: id.into(),
            path,
            channel,
            base_volume: base_volume.clamp(0.0, 1.0),
        }
    }

    /// Caminho como &str; retorna None se o path contiver bytes inválidos.
    pub fn path_str(&self) -> Option<&str> {
        self.path.to_str()
    }
}
