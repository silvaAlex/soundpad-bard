use anyhow::Context;
use image::RgbaImage;
use muda::MenuEvent;
use tray_icon::menu::{Menu, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub const MENU_SHOW: &str = "show_window";
pub const MENU_BARD_TOGGLE: &str = "bard_toggle";
pub const MENU_BARD_SKIP: &str = "bard_skip";
pub const MENU_QUIT: &str = "quit";

const ICON_PNG: &[u8] = include_bytes!("../../assets/icon.png");

pub enum TrayAction {
    ShowWindow,
    BardToggle,
    BardSkip,
    Quit,
    None,
}

pub fn build_tray() -> anyhow::Result<(TrayIcon, crossbeam_channel::Receiver<MenuEvent>)> {
    #[cfg(target_os = "linux")]
    {
        if !gtk::is_initialized() {
            let _ = gtk::init();
        }
    }

    let img: RgbaImage = image::load_from_memory(ICON_PNG)
        .context("failed to decode tray icon PNG")?
        .to_rgba8();
    let (width, height) = img.dimensions();
    let icon = Icon::from_rgba(img.into_raw(), width, height)
        .map_err(|e| anyhow::anyhow!("failed to create tray icon: {e}"))?;

    let menu = Menu::new();
    let show_item = MenuItem::with_id(MENU_SHOW, "Mostrar Janela", true, Option::None);
    let bard_toggle = MenuItem::with_id(MENU_BARD_TOGGLE, "Bard: Play/Pause", true, Option::None);
    let bard_skip = MenuItem::with_id(MENU_BARD_SKIP, "Bard: Skip", true, Option::None);
    let quit_item = MenuItem::with_id(MENU_QUIT, "Sair", true, Option::None);

    menu.append_items(&[
        &show_item,
        &muda::PredefinedMenuItem::separator(),
        &bard_toggle,
        &bard_skip,
        &muda::PredefinedMenuItem::separator(),
        &quit_item,
    ])?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .with_tooltip("Soundpad / Bard Minstrel")
        .build()?;

    let receiver = MenuEvent::receiver().clone();
    Ok((tray, receiver))
}

pub fn poll_menu_event(receiver: &crossbeam_channel::Receiver<MenuEvent>) -> TrayAction {
    match receiver.try_recv() {
        Ok(event) => {
            if event.id == MENU_SHOW {
                TrayAction::ShowWindow
            } else if event.id == MENU_BARD_TOGGLE {
                TrayAction::BardToggle
            } else if event.id == MENU_BARD_SKIP {
                TrayAction::BardSkip
            } else if event.id == MENU_QUIT {
                TrayAction::Quit
            } else {
                TrayAction::None
            }
        }
        Err(_) => TrayAction::None,
    }
}
