use super::audio_clip::{AudioChannel, AudioClip};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub schema_version: u32,
    pub obs: ObsConfig,
    pub soundpad: SoundpadConfig,
    pub bard: BardConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub sfx_source: String,
    pub bgm_source: String,
    pub ducking_filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundpadConfig {
    pub sfx_dir: PathBuf,
    pub clips: Vec<SoundpadClip>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundpadClip {
    pub id: String,
    pub filename: String,
    pub hotkey: String,
    pub volume: f32,
}

impl SoundpadClip {
    pub fn channel(&self) -> AudioChannel {
        AudioChannel::Sfx
    }

    pub fn to_audio_clip(&self, base_dir: &std::path::Path) -> AudioClip {
        AudioClip::new(
            &self.id,
            base_dir.join(&self.filename),
            self.channel(),
            self.volume,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BardConfig {
    pub music_dir: PathBuf,
    pub volume: f32,
    pub interval_secs: u64,
    pub enabled: bool,
    pub ducking: DuckingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuckingConfig {
    pub duck_ratio: f32,
    pub attack_ms: u32,
    pub release_ms: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            obs: ObsConfig {
                host: "localhost".into(),
                port: 4455,
                password: None,
                sfx_source: "SoundpadSFX".into(),
                bgm_source: "BardBGM".into(),
                ducking_filter: "SidechainDucking".into(),
            },
            soundpad: SoundpadConfig {
                sfx_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/sfx"),
                clips: Vec::new(),
            },
            bard: BardConfig {
                music_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/music"),
                volume: 0.3,
                interval_secs: 30,
                enabled: false,
                ducking: DuckingConfig {
                    duck_ratio: 0.25,
                    attack_ms: 100,
                    release_ms: 900,
                },
            },
        }
    }
}
