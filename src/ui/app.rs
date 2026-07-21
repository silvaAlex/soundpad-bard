use crate::application::ports::{ConfigRepository, HotkeyListener, ObsConnector};
use crate::application::use_cases::{ActivateBard, PauseBard, PlaySoundEffect, RegisterHotkey, SkipBardTrack};
use crate::domain::entities::{AudioChannel, BardState, SoundpadClip};
use crate::domain::services::HotkeyConflictChecker;
use crate::infrastructure::hotkeys::GlobalHotkeyListener;
use crate::infrastructure::obs::ObwsConnector;
use crate::infrastructure::persistence::JsonConfigRepository;
use crate::ui::tray::{self, TrayAction};
use crossbeam_channel::Receiver;
use eframe::egui;
use muda::MenuEvent;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use tray_icon::TrayIcon;

//ui/app

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
    pub bard_state: BardState,
    pub current_song: Option<String>,
    pub logs: Vec<String>,
    pub new_clip_filename: String,
    pub new_clip_hotkey: String,
    pub new_clip_volume: f32,
    pub runtime: tokio::runtime::Runtime,
    pub connector: Option<Arc<ObwsConnector>>,
    connect_rx: Option<mpsc::Receiver<Result<ObwsConnector, String>>>,

    hotkey_listener: Box<dyn HotkeyListener>,
    hotkey_checker: HotkeyConflictChecker,
    hotkey_to_clip: HashMap<u32, String>,
    bard_last_tick: Option<Instant>,

    _tray_icon: Option<TrayIcon>,
    tray_rx: Option<Receiver<MenuEvent>>,
    window_visible: bool,
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

        let (tray_icon, tray_rx) = match tray::build_tray() {
            Ok((icon, rx)) => (Some(icon), Some(rx)),
            Err(e) => {
                eprintln!("Falha ao criar system tray: {e}");
                (None, None)
            }
        };

        Self {
            tab: Tab::default(),
            config,
            obs_connected: false,
            bard_state: BardState::Idle,
            current_song: None,
            logs: Vec::new(),
            new_clip_filename: String::new(),
            new_clip_hotkey: String::new(),
            new_clip_volume: 0.8,
            runtime: tokio::runtime::Runtime::new().expect("falha ao criar tokio runtime"),
            connector: None,
            connect_rx: None,
            hotkey_listener,
            hotkey_checker: HotkeyConflictChecker::new(),
            hotkey_to_clip: HashMap::new(),
            bard_last_tick: None,
            _tray_icon: tray_icon,
            tray_rx,
            window_visible: true,
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
            bard_state: BardState::Idle,
            current_song: None,
            logs: Vec::new(),
            new_clip_filename: String::new(),
            new_clip_hotkey: String::new(),
            new_clip_volume: 0.8,
            runtime: tokio::runtime::Runtime::new().expect("falha ao criar tokio runtime"),
            connector: None,
            connect_rx: None,
            hotkey_listener: listener,
            hotkey_checker: HotkeyConflictChecker::new(),
            hotkey_to_clip: HashMap::new(),
            bard_last_tick: None,
            _tray_icon: None,
            tray_rx: None,
            window_visible: true,
        }
    }

    // ── OBS connection ─────────────────────────────────────────────

    pub(crate) fn start_connect(&mut self) {
        self.add_log("🔗 Tentando conectar ao OBS...".into());
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
                    self.connector = Some(Arc::new(connector));
                    self.obs_connected = true;
                    self.add_log("✅ Conectado ao OBS com sucesso!".into());
                    self.sync_hotkeys();
                }
                Ok(Err(e)) => {
                    self.add_log(format!("❌ Erro ao conectar: {e}"));
                    self.connect_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.add_log("❌ Canal de conexão fechado inesperadamente".into());
                    self.connect_rx = None;
                }
            }
        }
    }

    pub(crate) fn disconnect(&mut self) {
        self.unregister_all_hotkeys();
        self.connector = None;
        self.obs_connected = false;
        self.bard_state = BardState::Idle;
        self.add_log("🔌 Desconectado do OBS".into());
    }

    // ── Config persistence ─────────────────────────────────────────

    pub(crate) fn save_config(&mut self) {
        let repo = JsonConfigRepository::new(JsonConfigRepository::default_path());
        if let Err(e) = repo.save(&self.config) {
            self.add_log(format!("⚠️  Erro ao salvar config: {e}"));
        }
    }

    pub(crate) fn add_log(&mut self, msg: String) {
        self.logs.push(msg);
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    // ── Hotkey management (COM VERIFICAÇÃO DE CONFLITOS) ──────────

    /// Sincronizar hotkeys com a configuração
    /// Desregistra todas as antigas e registra as novas
    pub(crate) fn sync_hotkeys(&mut self) {
        self.unregister_all_hotkeys();
        self.add_log("🔄 Sincronizando hotkeys...".into());

        let clips: Vec<SoundpadClip> = self.config.soundpad.clips.clone();

        let mut success_count = 0;
        let mut conflict_count = 0;

        for clip in &clips {
            if let Ok(new_hk) = crate::domain::entities::Hotkey::new(&clip.hotkey) {
                let duplicate = clips.iter().filter(|c| c.id != clip.id).any(|other| {
                    crate::domain::entities::Hotkey::new(&other.hotkey)
                        .map(|other_hk| new_hk.conflict_with(&other_hk))
                        .unwrap_or(false)
                });
                if duplicate {
                    conflict_count += 1;
                    self.add_log(format!(
                        "❌ Clip '{}' ({}) - hotkey duplicada entre clips",
                        clip.filename, clip.hotkey
                    ));
                    continue;
                }
            }

            match self.register_clip_hotkey(clip) {
                Ok(()) => success_count += 1,
                Err(e) => {
                    conflict_count += 1;
                    self.add_log(format!(
                        "❌ Clip '{}' ({}) - {}",
                        clip.filename, clip.hotkey, e
                    ));
                }
            }
        }

        self.add_log(format!(
            "📊 Resultado: {} registradas, {} com conflito",
            success_count, conflict_count
        ));
    }

    /// Registrar uma hotkey de clip
    ///
    /// # Processo
    /// 1. Parse e normaliza a hotkey string
    /// 2. Verifica conflito com hotkeys existentes
    /// 3. Registra no listener (que registra no SO)
    /// 4. Rastreia para polling e limpeza
    fn register_clip_hotkey(&mut self, clip: &SoundpadClip) -> anyhow::Result<()> {
        let mut register_hotkey = RegisterHotkey {
            listener: &mut *self.hotkey_listener,
            checker: &mut self.hotkey_checker,
        };
        let id = register_hotkey.execute(&clip.hotkey)?;
        self.hotkey_to_clip.insert(id, clip.id.clone());
        Ok(())
    }

    /// Desregistrar uma hotkey específica
    pub(crate) fn unregister_clip_hotkey(&mut self, clip: &SoundpadClip) {
        let hotkey = match crate::domain::entities::Hotkey::new(&clip.hotkey) {
            Ok(hk) => hk,
            Err(_) => return,
        };
        self.hotkey_checker.unregister(&hotkey);
        let ids_to_remove: Vec<u32> = self
            .hotkey_to_clip
            .iter()
            .filter(|(_, id)| *id == &clip.id)
            .map(|(hk_id, _)| *hk_id)
            .collect();
        for id in ids_to_remove {
            self.hotkey_to_clip.remove(&id);
            let _ = self.hotkey_listener.unregister(id);
        }
    }

    /// Desregistrar todas as hotkeys
    fn unregister_all_hotkeys(&mut self) {
        if self.hotkey_to_clip.is_empty() {
            return;
        }

        let ids_to_remove: Vec<u32> = self.hotkey_to_clip.keys().copied().collect();

        for id in ids_to_remove {
            if let Err(e) = self.hotkey_listener.unregister(id) {
                self.add_log(format!("⚠️  Erro ao desregistrar hotkey {}: {}", id, e));
            }
        }

        self.hotkey_to_clip.clear();
        self.hotkey_checker.clear();
    }

    // ── Hotkey event polling (called every frame) ──────────────────

    pub(crate) fn poll_hotkey_events(&mut self) {
        let fired = self.hotkey_listener.poll_events();
        for id in fired {
            if let Some(clip_id) = self.hotkey_to_clip.get(&id).cloned() {
                self.play_clip_by_id(&clip_id);
            }
        }
    }

    // ── SFX actions ────────────────────────────────────────────────

    pub(crate) fn play_clip_by_id(&mut self, clip_id: &str) {
        let Some(connector) = self.connector.clone() else {
            self.add_log("⚠️  OBS não conectado".into());
            return;
        };

        let Some(clip) = self.config.soundpad.clips.iter().find(|c| c.id == clip_id) else {
            self.add_log(format!("❌ Clip '{}' não encontrado", clip_id));
            return;
        };

        if self.bard_state == BardState::Playing {
            self.bard_state = BardState::Ducked;
        }

        let config = self.config.clone();
        let clip_name = clip.filename.clone();
        let clip_id_owned = clip_id.to_owned();

        self.runtime.handle().spawn(async move {
            let uc = PlaySoundEffect {
                obs: &*connector,
                config: &config,
            };
            if let Err(e) = uc.execute(&clip_id_owned).await {
                eprintln!("[SFX] {e}");
            }
        });

        self.add_log(format!("🔊 SFX: {}", clip_name));
    }

    // ── Bard actions ───────────────────────────────────────────────

    pub(crate) fn bard_toggle(&mut self) {
        if !self.obs_connected {
            self.add_log("⚠️  OBS não conectado".into());
            return;
        }
        let next = match self.bard_state {
            BardState::Idle | BardState::Paused => BardState::Playing,
            BardState::Playing | BardState::Ducked => BardState::Paused,
        };
        if !self.bard_state.can_transition_to(next) {
            self.add_log(format!(
                "⚠️  Transição inválida: {} → {}",
                self.bard_state, next
            ));
            return;
        }
        self.bard_state = next;
        match self.bard_state {
            BardState::Playing => {
                self.bard_last_tick = Some(Instant::now());
                self.bard_play_random();
                self.add_log("🎵 Bard iniciado".into());
            }
            BardState::Paused => {
                self.bard_pause();
                self.bard_last_tick = None;
                self.add_log("⏸️  Bard pausado".into());
            }
            _ => {}
        }
    }

    pub(crate) fn bard_skip(&mut self) {
        if !matches!(self.bard_state, BardState::Playing | BardState::Ducked) {
            return;
        }

        let Some(connector) = self.connector.clone() else {
            self.add_log("⚠️  OBS não conectado".into());
            return;
        };

        let dir = &self.config.bard.music_dir;
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                self.add_log(format!("❌ Falha ao ler pasta de músicas: {e}"));
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
            self.add_log("❌ Nenhum .wav encontrado na pasta de músicas".into());
            return;
        }

        let chosen = {
            let mut rng = rand::thread_rng();
            wavs.choose(&mut rng).unwrap()
        };

        let file_path = chosen.path();
        let Some(file_str) = file_path.to_str().map(|s| s.to_owned()) else {
            self.add_log("❌ Caminho da música inválido".into());
            return;
        };

        let song_name = file_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_str.clone());

        self.current_song = Some(song_name.clone());
        self.add_log(format!("🎶 BGM [{}]: {}", AudioChannel::Bgm, song_name));

        let config = self.config.clone();
        self.runtime.handle().spawn(async move {
            let uc = SkipBardTrack {
                obs: &*connector,
                config: &config,
            };
            if let Err(e) = uc.execute(&file_str).await {
                eprintln!("[BGM skip] {e}");
            }
        });
    }

    fn bard_play_random(&mut self) {
        let Some(connector) = self.connector.clone() else {
            self.add_log("⚠️  OBS não conectado".into());
            return;
        };

        if self.bard_state == BardState::Ducked {
            self.bard_state = BardState::Playing;
        }

        let dir = &self.config.bard.music_dir;
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                self.add_log(format!("❌ Falha ao ler pasta de músicas: {e}"));
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
            self.add_log("❌ Nenhum .wav encontrado na pasta de músicas".into());
            return;
        }

        let chosen = {
            let mut rng = rand::thread_rng();
            wavs.choose(&mut rng).unwrap()
        };

        let file_path = chosen.path();
        let Some(file_str) = file_path.to_str().map(|s| s.to_owned()) else {
            self.add_log("❌ Caminho da música inválido".into());
            return;
        };

        let song_name = file_path
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_str.clone());

        self.current_song = Some(song_name.clone());
        self.add_log(format!("🎶 BGM [{}]: {}", AudioChannel::Bgm, song_name));

        let config = self.config.clone();
        self.runtime.handle().spawn(async move {
            let uc = ActivateBard {
                obs: &*connector,
                config: &config,
            };
            if let Err(e) = uc.execute(&file_str).await {
                eprintln!("[BGM] {e}");
            }
        });
    }

    fn bard_pause(&self) {
        let Some(connector) = self.connector.clone() else {
            return;
        };

        let config = self.config.clone();
        self.runtime.handle().spawn(async move {
            let uc = PauseBard {
                obs: &*connector,
                config: &config,
            };
            let _ = uc.execute().await;
        });
    }

    // ── Tray event polling ──────────────────────────────────────────

    pub(crate) fn poll_tray_events(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.tray_rx else {
            return;
        };

        match tray::poll_menu_event(rx) {
            TrayAction::ShowWindow => {
                self.window_visible = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            TrayAction::BardToggle => self.bard_toggle(),
            TrayAction::BardSkip => self.bard_skip(),
            TrayAction::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            TrayAction::None => {}
        }
    }

    pub(crate) fn poll_bard_scheduler(&mut self) {
        if !matches!(self.bard_state, BardState::Playing | BardState::Ducked)
            || !self.config.bard.enabled
        {
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
        self.poll_tray_events(ui.ctx());

        let ctx = ui.ctx();
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.window_visible = false;
        }

        egui::Panel::top("tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Soundpad, "Soundpad");
                ui.selectable_value(&mut self.tab, Tab::Bard, "Bard Minstrel");
                ui.selectable_value(&mut self.tab, Tab::Settings, "Configurações");
            });
        });

        egui::Panel::bottom("status").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let status = if self.obs_connected {
                    "✅ OBS: Conectado"
                } else {
                    "❌ OBS: Desconectado"
                };
                ui.label(status);
                ui.separator();
                if let Some(ref song) = self.current_song {
                    ui.label(format!("🎶 Tocando: {song}"));
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

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHotkeyListener;

    impl HotkeyListener for MockHotkeyListener {
        fn register(&mut self, _hotkey: &crate::domain::entities::Hotkey) -> anyhow::Result<u32> {
            Ok(1)
        }

        fn unregister(&mut self, _id: u32) -> anyhow::Result<()> {
            Ok(())
        }

        fn poll_events(&mut self) -> Vec<u32> {
            Vec::new()
        }
    }

    #[test]
    fn test_register_clip_hotkey() {
        let mut app = SoundpadApp::new_with_listener(Box::new(MockHotkeyListener));
        let clip = SoundpadClip {
            id: "clip1".to_string(),
            filename: "beep.wav".to_string(),
            hotkey: "Ctrl+Alt+A".to_string(),
            volume: 0.8,
        };

        app.register_clip_hotkey(&clip).unwrap();
        assert!(app.hotkey_to_clip.values().any(|id| id == "clip1"));
    }

    #[test]
    fn test_invalid_hotkey_format() {
        let mut app = SoundpadApp::new_with_listener(Box::new(MockHotkeyListener));
        let clip = SoundpadClip {
            id: "bad_clip".to_string(),
            filename: "beep.wav".to_string(),
            hotkey: "INVALID".to_string(),
            volume: 0.8,
        };

        let result = app.register_clip_hotkey(&clip);
        assert!(result.is_err());
    }
}
