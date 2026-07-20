use crate::ui::app::SoundpadApp;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, app: &mut SoundpadApp) {
    ui.heading("Configuracoes - OBS WebSocket");
    ui.separator();

    ui.horizontal(|ui| {
        ui.label("Host:");
        ui.text_edit_singleline(&mut app.config.obs.host);
    });

    ui.horizontal(|ui| {
        ui.label("Porta:");
        ui.add(egui::DragValue::new(&mut app.config.obs.port).speed(1));
    });

    ui.horizontal(|ui| {
        ui.label("Senha:");
        let mut pwd = app.config.obs.password.clone().unwrap_or_default();
        let response = ui.add(egui::TextEdit::singleline(&mut pwd).password(true));
        if response.changed() {
            app.config.obs.password = if pwd.is_empty() { None } else { Some(pwd) };
        }
    });

    ui.separator();
    ui.label("Sources no OBS:");
    ui.horizontal(|ui| {
        ui.label("SFX Source:");
        ui.text_edit_singleline(&mut app.config.obs.sfx_source);
    });
    ui.horizontal(|ui| {
        ui.label("BGM Source:");
        ui.text_edit_singleline(&mut app.config.obs.bgm_source);
    });
    ui.horizontal(|ui| {
        ui.label("Filtro Ducking:");
        ui.text_edit_singleline(&mut app.config.obs.ducking_filter);
    });

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Conectar ao OBS").clicked() {
            app.start_connect();
        }
        if ui.button("Desconectar").clicked() {
            app.disconnect();
        }
    });

    if ui.button("Salvar Configuracao").clicked() {
        app.save_config();
        app.add_log("Configuracao salva".into());
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
