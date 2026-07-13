mod application;
mod domain;
mod infrastructure;
mod ui;

use ui::SoundpadApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([700.0, 500.0])
            .with_title("Soundpad / Bard Minstrel"),
        ..Default::default()
    };

    eframe::run_native(
        "Soundpad / Bard Minstrel",
        native_options,
        Box::new(|cc| Ok(Box::new(SoundpadApp::new(cc)))),
    )
}
