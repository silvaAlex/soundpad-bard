use crate::application::ports::{ConfigRepository, HotkeyListener, ObsConnector};
use crate::domain::entities::SoundpadClip;
use crate::infrastructure::hotkeys::GlobalHotkeyListener;
use crate::infrastructure::obs::ObwsConnector;
use crate::infrastructure::persistence::JsonConfigRepository;
use eframe::egui;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Default, PartialEq)]
pub enum Tab {
    #[default]
    Soundpad,
    Bard,
    Settings,
}

pub struct SoundpadApp {
    pub tab: Tab,
    pub config: crate::domain::entities::AppConfig,
    pub obs_connected: bool,
    pub bard_playing: bool,
    pub current_song: Option<String>,
    pub logs: Vec<String>,
    pub new_clip_filename: String,
    pub new_clip_hotkey: String,
    pub new_clip_volume: f32,
    pub runtime: tokio::runtime::Runtime,
    pub connector: Option<Arc<Mutex<ObwsConnector>>>,
    connect_rx: Option<mpsc::Receiver<Result<ObwsConnector, String>>>,

    hotkey_listener: Box<dyn HotkeyListener>,
    registered_hotkeys: HashMap<String, u32>,
    hotkey_to_clip: HashMap<u32, String>,
    bard_last_tick: Option<Instant>,
}

impl SoundpadApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let repo = JsonConfigRepository::new(JsonConfigRepository::default_path());
        let config = repo.load().unwrap_or_default();

        let hotkey_listener: Box<dyn HotkeyListener> = match GlobalHotkeyListener::new() {
            Ok(listener) => Box::new(listener),
            Err(e) => {
                eprintln!("Falha ao criar listener de hotkeys: {}", e);
                Box::new(GlobalHotkeyListener::default())
            }
        };

        Self {
            tab: Tab::default(),
            config,
            obs_connected: false,
            bard_playing: false,
            current_song: None,
            logs: Vec::new(),
            new_clip_filename: String::new(),
            new_clip_hotkey: String::new(),
            new_clip_volume: 0.8,
            runtime: tokio::runtime::Runtime::new().expect("falha ao criar tokio runtime"),
            connector: None,
            connect_rx: None,
            hotkey_listener,
            registered_hotkeys: HashMap::new(),
            hotkey_to_clip: HashMap::new(),
            bard_last_tick: None,
        }
    }

    #[cfg(test)]
    pub fn new_with_listener(listener: Box<dyn HotkeyListener>) -> Self {
        let repo = JsonConfigRepository::new(JsonConfigRepository::default_path());
        let config = repo.load().unwrap_or_default();
 
        Self {
            tab: Tab::default(),
            config,
            obs_connected: false,
            bard_playing: false,
            current_song: None,
            logs: Vec::new(),
            new_clip_filename: String::new(),
            new_clip_hotkey: String::new(),
            new_clip_volume: 0.8,
            runtime: tokio::runtime::Runtime::new()
                .expect("falha ao criar tokio runtime"),
            connector: None,
            connect_rx: None,
            hotkey_listener: listener,
            registered_hotkeys: HashMap::new(),
            hotkey_to_clip: HashMap::new(),
            bard_last_tick: None,
        }
    }


    // ── OBS connection ─────────────────────────────────────────────

    pub(crate) fn start_connect(&mut self) {
        self.add_log("Tentando conectar ao OBS...".into());
        let connector = ObwsConnector::new(
            self.config.obs.host.clone(),
            self.config.obs.port,
            self.config.obs.password.clone(),
        );
        let (tx, rx) = mpsc::channel();
        let handle = self.runtime.handle().clone();
        handle.spawn(async move {
            let mut c = connector;
            match c.connect().await {
                Ok(()) => {
                    let _ = tx.send(Ok(c));
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });
        self.connect_rx = Some(rx);
    }

    pub(crate) fn poll_connect_result(&mut self) {
        if let Some(rx) = &self.connect_rx {
            match rx.try_recv() {
                Ok(Ok(connector)) => {
                    self.connector = Some(Arc::new(Mutex::new(connector)));
                    self.obs_connected = true;
                    self.add_log("Conectado ao OBS com sucesso!".into());
                    self.sync_hotkeys();
                }
                Ok(Err(e)) => {
                    self.add_log(format!("Erro ao conectar: {e}"));
                    self.connect_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.add_log("Canal de conexao fechado inesperadamente".into());
                    self.connect_rx = None;
                }
            }
        }
    }

    pub(crate) fn disconnect(&mut self) {
        self.unregister_all_hotkeys();
        self.connector = None;
        self.obs_connected = false;
        self.add_log("Desconectado do OBS".into());
    }

    // ── Config persistence ─────────────────────────────────────────

    pub(crate) fn save_config(&mut self) {
        let repo = JsonConfigRepository::new(JsonConfigRepository::default_path());
        if let Err(e) = repo.save(&self.config) {
            self.add_log(format!("erro ao salvar config: {e}"));
        }
    }

    pub(crate) fn add_log(&mut self, msg: String) {
        self.logs.push(msg);
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    // ── Hotkey management ──────────────────────────────────────────

    pub(crate) fn sync_hotkeys(&mut self) {
        self.unregister_all_hotkeys();
 
        let clips: Vec<SoundpadClip> = self.config.soundpad.clips.clone();
        for clip in &clips {
            if let Err(e) = self.register_clip_hotkey(clip) {
                self.add_log(format!(
                    "falha ao registrar hotkey '{}': {e}",
                    clip.hotkey
                ));
            }
        }
    }

    fn register_clip_hotkey(&mut self, clip: &SoundpadClip) -> anyhow::Result<()> {

        let hotkey = crate::domain::entities::Hotkey::new(&clip.hotkey);
        
        let id = self.hotkey_listener.register(&hotkey)
            .map_err(|e| anyhow::anyhow!(
                "falha ao registrar hotkey '{}': {e}", 
                clip.hotkey
            ))?;

        self.registered_hotkeys.insert(clip.id.clone(), id);
        self.hotkey_to_clip.insert(id, clip.id.clone());
        Ok(())
    }

    fn unregister_all_hotkeys(&mut self) {
        // ✅ MUDANÇA: Iterar IDs e desregistrar via listener
        let ids_to_remove: Vec<u32> = self.registered_hotkeys.values().copied().collect();
        
        for id in ids_to_remove {
            if let Err(e) = self.hotkey_listener.unregister(id) {
                self.add_log(format!("erro ao desregistrar hotkey {}: {e}", id));
            }
        }
 
        self.registered_hotkeys.clear();
        self.hotkey_to_clip.clear();
    }


    // ── Hotkey event polling (called every frame) ──────────────────

    pub(crate) fn poll_hotkey_events(&mut self) {
        let fired = self.hotkey_listener.poll_events();
        let to_play: Vec<String> = fired
            .iter()
            .filter_map(|id| self.hotkey_to_clip.get(id).cloned())
            .collect();

        for clip_id in to_play {
            self.play_clip_by_id(&clip_id);
        }
    }

    // ── SFX actions ────────────────────────────────────────────────

    pub(crate) fn play_clip_by_id(&mut self, clip_id: &str) {
        let Some(connector) = self.connector.clone() else {
            self.add_log("OBS nao conectado".into());
            return;
        };

        let Some(clip) = self.config.soundpad.clips.iter().find(|c| c.id == clip_id) else {
            self.add_log(format!("clip '{clip_id}' nao encontrado"));
            return;
        };

        let file_path = self.config.soundpad.sfx_dir.join(&clip.filename);
        let Some(file_str) = file_path.to_str().map(|s| s.to_owned()) else {
            self.add_log("caminho do arquivo invalido".into());
            return;
        };

        let source = self.config.obs.sfx_source.clone();
        let volume = clip.volume;
        let bgm_source = self.config.obs.bgm_source.clone();
        let ducking_filter = self.config.obs.ducking_filter.clone();
        let ducking_ms = self.config.bard.ducking.release_ms as u64;
        let clip_name = clip.filename.clone();

        self.runtime.handle().spawn(async move {
            let obs = connector.lock().await;
            let _ = obs.set_source_volume(&source, volume).await;
            if let Err(e) = obs.play_sfx(&source, &file_str).await {
                eprintln!("[SFX] {e}");
                return;
            }
            let _ = obs
                .set_ducking_filter_enabled(&bgm_source, &ducking_filter, true)
                .await;

            // Libera o lock durante o sleep para não bloquear outras operações
            drop(obs);
            tokio::time::sleep(Duration::from_millis(ducking_ms)).await;

            let obs = connector.lock().await;
            let _ = obs
                .set_ducking_filter_enabled(&bgm_source, &ducking_filter, false)
                .await;
        });

        self.add_log(format!("SFX: {clip_name}"));
    }

    // ── Bard actions ───────────────────────────────────────────────

    pub(crate) fn bard_toggle(&mut self) {
        if !self.obs_connected {
            self.add_log("OBS nao conectado".into());
            return;
        }
        self.bard_playing = !self.bard_playing;
        if self.bard_playing {
            self.bard_last_tick = Some(Instant::now());
            self.bard_play_random();
            self.add_log("Bard iniciado".into());
        } else {
            self.bard_pause();
            self.bard_last_tick = None;
            self.add_log("Bard pausado".into());
        }
    }

    pub(crate) fn bard_skip(&mut self) {
        if self.bard_playing {
            self.bard_play_random();
        }
    }

    fn bard_play_random(&mut self) {
        let Some(connector) = self.connector.clone() else {
            self.add_log("OBS nao conectado".into());
            return;
        };

        let dir = &self.config.bard.music_dir;
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                self.add_log(format!("falha ao ler pasta de musicas: {e}"));
                return;
            }
        };

        let wavs: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "wav")
                    .unwrap_or(false)
            })
            .collect();

        if wavs.is_empty() {
            self.add_log("nenhum .wav encontrado na pasta de musicas".into());
            return;
        }

        let chosen = {
            let mut rng = rand::thread_rng();
            wavs.choose(&mut rng).unwrap()
        };

        let file_path = chosen.path();
        let Some(file_str) = file_path.to_str().map(|s| s.to_owned()) else {
            self.add_log("caminho da musica invalido".into());
            return;
        };

        let song_name = file_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_str.clone());

        self.current_song = Some(song_name.clone());
        self.add_log(format!("BGM: {song_name}"));

        let source = self.config.obs.bgm_source.clone();
        let volume = self.config.bard.volume;

        self.runtime.handle().spawn(async move {
            let obs = connector.lock().await;
            let _ = obs.set_source_volume(&source, volume).await;
            if let Err(e) = obs.play_sfx(&source, &file_str).await {
                eprintln!("[BGM] {e}");
            }
        });
    }

    fn bard_pause(&self) {
        let Some(connector) = self.connector.clone() else {
            return;
        };
        let source = self.config.obs.bgm_source.clone();

        self.runtime.handle().spawn(async move {
            let obs = connector.lock().await;
            let _ = obs.set_source_volume(&source, 0.0).await;
        });
    }

    pub(crate) fn poll_bard_scheduler(&mut self) {
        if !self.bard_playing || !self.config.bard.enabled {
            self.bard_last_tick = None;
            return;
        }

        let now = Instant::now();
        let last = self.bard_last_tick.unwrap_or(now);
        let interval = Duration::from_secs(self.config.bard.interval_secs);

        if now.duration_since(last) >= interval {
            self.bard_last_tick = Some(now);
            self.bard_play_random();
        }
    }
}

impl eframe::App for SoundpadApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_connect_result();
        self.poll_hotkey_events();
        self.poll_bard_scheduler();

        egui::Panel::top("tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Soundpad, "Soundpad");
                ui.selectable_value(&mut self.tab, Tab::Bard, "Bard Minstrel");
                ui.selectable_value(&mut self.tab, Tab::Settings, "Configuracoes");
            });
        });

        egui::Panel::bottom("status").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let status = if self.obs_connected {
                    "OBS: Conectado"
                } else {
                    "OBS: Desconectado"
                };
                ui.label(status);
                ui.separator();
                if let Some(ref song) = self.current_song {
                    ui.label(format!("Tocando: {song}"));
                }
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.tab {
            Tab::Soundpad => crate::ui::views::soundpad_view::show(ui, self),
            Tab::Bard => crate::ui::views::bard_view::show(ui, self),
            Tab::Settings => crate::ui::views::settings_view::show(ui, self),
        });
    }
}
