use eframe::egui::{self, CentralPanel, ComboBox};
use std::fs;
use std::path::Path;
use std::io::Write;
use egui::ViewportCommand;

// Struttura del Config per toml
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Config {
    source_path: String,
    destination_path: String,
    backup_type: String,
    extensions_to_backup: Vec<String>,
}

// La GUI dell'applicazione
#[derive(Default)]
struct BackupApp {
    source_path: String,
    destination_path: String,
    backup_type: String,
    extensions_to_backup: String, // Le estensioni sono inserite come stringa, verranno separate dopo
}

impl BackupApp {
    // Metodo per salvare il file di configurazione
    fn save_config(&self) {
        let config = Config {
            source_path: self.source_path.clone(),
            destination_path: self.destination_path.clone(),
            backup_type: self.backup_type.clone(),
            extensions_to_backup: self.extensions_to_backup
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        };
        let toml_str = toml::to_string(&config).unwrap();
        let mut file = fs::File::create("config.toml").unwrap();
        file.write_all(toml_str.as_bytes()).unwrap();
    }
}

impl eframe::App for BackupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Backup Configuration");

            // Campo per selezionare il percorso sorgente
            ui.label("Source Path:");
            ui.text_edit_singleline(&mut self.source_path);

            // Campo per selezionare il percorso destinazione
            ui.label("Destination Path:");
            ui.text_edit_singleline(&mut self.destination_path);

            // ComboBox per scegliere il tipo di backup
            ui.label("Backup Type:");
            ComboBox::from_label("")
                .selected_text(&self.backup_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.backup_type, "full-disk".to_string(), "Full Disk");
                    ui.selectable_value(&mut self.backup_type, "directory".to_string(), "Directory");
                    ui.selectable_value(&mut self.backup_type, "selective".to_string(), "Selective");
                });

            // Campo per inserire le estensioni da includere nel backup
            ui.label("File Extensions (comma separated):");
            ui.text_edit_singleline(&mut self.extensions_to_backup);

            // Bottone per chiudere e salvare la configurazione
            if ui.button("Save and Exit").clicked() {
                self.save_config();
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
        });
    }
}

// Funzione per avviare la GUI solo se `config.toml` non esiste
pub fn show_gui_if_needed() -> Result<(), eframe::Error> {
    if !Path::new("config.toml").exists() {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([400f32, 250f32]),
            ..Default::default()
        };
        eframe::run_native(
            "Backup Configuration",
            options,
            Box::new(|_cc| Ok(Box::new(BackupApp::default()))),
        )
    } else {
        println!("Il file config.toml esiste già, la GUI non verrà mostrata.");
        Ok(())
    }
}
