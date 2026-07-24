mod application;
mod domain;
mod infrastructure;
mod ui;

use ui::SoundpadApp;
use eframe::egui::IconData;

fn load_icon() -> IconData {
    let image = image::load_from_memory(include_bytes!("../assets/icon-1024.png"))
        .unwrap()
        .into_rgba8();

    let (width, height) = image.dimensions();

    IconData {
        rgba: image.into_raw(),
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_title("Soundpad / Bard Minstrel")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Soundpad / Bard Minstrel",
        native_options,
        Box::new(|cc| Ok(Box::new(SoundpadApp::new(cc)))),
    )
}
