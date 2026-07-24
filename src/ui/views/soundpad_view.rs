use crate::ui::app::SoundpadApp;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, app: &mut SoundpadApp) {
    ui.heading("Soundpad - Efeitos Sonoros");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Pasta SFX:");
        ui.label(app.config.soundpad.sfx_dir.display().to_string());
        if ui.button("Selecionar Pasta").clicked() {
            if let Some(dir) = rfd::FileDialog::new()
                .set_title("Selecionar pasta de efeitos sonoros")
                .pick_folder()
            {
                app.config.soundpad.sfx_dir = dir;
                app.save_config();
            }
        }
    });

    ui.separator();
    ui.label("Adicionar novo clip:");
    ui.horizontal(|ui| {
        ui.label("Arquivo:");
        ui.text_edit_singleline(&mut app.new_clip_filename);
        if ui.button("Procurar").clicked() {
            if let Some(file) = rfd::FileDialog::new()
                .set_title("Selecionar arquivo .wav")
                .add_filter("WAV", &["wav"])
                .pick_file()
            {
                if let Some(name) = file.file_name().map(|n| n.to_string_lossy().to_string()) {
                    app.new_clip_filename = name;
                }
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Atalho:");
        ui.text_edit_singleline(&mut app.new_clip_hotkey);
        ui.label("Volume:");
        ui.add(egui::Slider::new(&mut app.new_clip_volume, 0.0..=1.0).text("vol"));
        if ui.button("Adicionar").clicked()
            && !app.new_clip_filename.is_empty()
            && !app.new_clip_hotkey.is_empty()
        {
            let id = format!("clip_{}", app.config.soundpad.clips.len());
            app.config
                .soundpad
                .clips
                .push(crate::domain::entities::SoundpadClip {
                    id,
                    filename: app.new_clip_filename.clone(),
                    hotkey: app.new_clip_hotkey.clone(),
                    volume: app.new_clip_volume,
                });
            app.new_clip_filename.clear();
            app.new_clip_hotkey.clear();
            app.new_clip_volume = 0.8;
            app.save_config();
            app.sync_hotkeys();
        }
    });

    ui.separator();
    ui.label("Clips registrados:");

    let mut play_id: Option<String> = None;
    let mut remove_indices = Vec::new();

    for (i, clip) in app.config.soundpad.clips.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(&clip.filename);
            ui.label(format!("[{}]", clip.hotkey));
            ui.label(format!("{:.0}%", clip.volume * 100.0));
            if ui.button("Tocar").clicked() {
                play_id = Some(clip.id.clone());
            }
            if ui.button("Remover").clicked() {
                remove_indices.push(i);
            }
        });
    }

    if let Some(clip_id) = play_id {
        app.play_clip_by_id(&clip_id);
    }

    for i in remove_indices.into_iter().rev() {
        let removed = app.config.soundpad.clips.remove(i);
        app.unregister_clip_hotkey(&removed);
        app.save_config();
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
