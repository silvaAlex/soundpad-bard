use crate::domain::entities::BardState;
use crate::ui::app::SoundpadApp;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, app: &mut SoundpadApp) {
    ui.heading("Bard Minstrel - Musica de Fundo");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Pasta de musicas:");
        ui.label(app.config.bard.music_dir.display().to_string());
        if ui.button("Selecionar Pasta").clicked() {
            if let Some(dir) = rfd::FileDialog::new()
                .set_title("Selecionar pasta de musicas")
                .pick_folder()
            {
                app.config.bard.music_dir = dir;
                app.save_config();
            }
        }
    });

    ui.separator();

    ui.horizontal(|ui| {
        let is_active = matches!(app.bard_state, BardState::Playing | BardState::Ducked);
        let btn_label = if is_active { "Pausar" } else { "Tocar" };
        if ui.button(btn_label).clicked() {
            app.bard_toggle();
        }
        if ui.button("Proxima").clicked() {
            app.bard_skip();
        }
        ui.separator();
        ui.label(format!("Estado: {}", app.bard_state));
    });

    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Habilitado:");
        ui.checkbox(&mut app.config.bard.enabled, "");
    });

    ui.horizontal(|ui| {
        ui.label("Volume:");
        ui.add(
            egui::Slider::new(&mut app.config.bard.volume, 0.0..=1.0)
                .text("vol")
                .clamping(egui::SliderClamping::Never),
        );
    });

    ui.horizontal(|ui| {
        ui.label("Intervalo (seg):");
        ui.add(
            egui::Slider::new(&mut app.config.bard.interval_secs, 5..=300)
                .text("s")
                .clamping(egui::SliderClamping::Never),
        );
    });

    ui.separator();
    ui.label("Ducking:");
    ui.horizontal(|ui| {
        ui.label("Ratio:");
        ui.add(
            egui::Slider::new(&mut app.config.bard.ducking.duck_ratio, 0.05..=0.9)
                .text("ratio")
                .clamping(egui::SliderClamping::Never),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Attack (ms):");
        ui.add(
            egui::Slider::new(&mut app.config.bard.ducking.attack_ms, 10..=500)
                .text("ms")
                .clamping(egui::SliderClamping::Never),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Release (ms):");
        ui.add(
            egui::Slider::new(&mut app.config.bard.ducking.release_ms, 100..=3000)
                .text("ms")
                .clamping(egui::SliderClamping::Never),
        );
    });

    if ui.button("Salvar Configuracao").clicked() {
        app.save_config();
        app.add_log("Configuracao do Bard salva".into());
    }

    if !app.logs.is_empty() {
        ui.separator();
        ui.label("Log:");
        egui::ScrollArea::vertical()
            .max_height(100.0)
            .show(ui, |ui| {
                for log in app.logs.iter().rev().take(20) {
                    ui.label(log);
                }
            });
    }
}
