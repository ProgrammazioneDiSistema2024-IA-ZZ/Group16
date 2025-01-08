use eframe::egui::{self, CentralPanel, ComboBox};
use std::{env, fs};
use std::path::{Path, PathBuf};
use std::io::Write;
use std::sync::{Arc, Mutex};
use eframe::Frame;
use egui::{Align, Color32, Context, Layout, RichText, ViewportCommand, Window};
use rfd::FileDialog;

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
struct ConfigWindow {
    source_path: String,
    destination_path: String,
    backup_type: String, // Default sarà impostato a "full-disk" manualmente
    extensions_to_backup: String, // Le estensioni sono inserite come stringa, verranno separate dopo
}

impl ConfigWindow {
    // Metodo per salvare il file di configurazione
    fn save_config(&self, config_file_path: PathBuf) {
        println!("{:?}", self.backup_type);
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
        let mut file = fs::File::create(config_file_path.join("config.toml")).unwrap();
        file.write_all(toml_str.as_bytes()).unwrap();
    }

    // Metodo per selezionare una directory tramite un file dialog
    fn select_directory() -> Option<String> {
        FileDialog::new()
            .pick_folder()  // Apre il dialogo per selezionare una cartella
            .map(|path| path.display().to_string())  // Converte il percorso selezionato in stringa
    }
}

impl eframe::App for ConfigWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
        let config_file_path = exe_path.parent().unwrap().join("Resources/");

        // Variabile per tracciare gli errori
        let mut error_message = String::new();

        // Funzione per controllare la validità dei campi
        let is_valid = !self.source_path.trim().is_empty()
            && !self.destination_path.trim().is_empty()
            && !self.backup_type.trim().is_empty()
            && (self.backup_type != "selective" || !self.extensions_to_backup.trim().is_empty())
            && self.source_path != self.destination_path; // Controllo sui percorsi

        CentralPanel::default().show(ctx, |ui| {
            // Spaziatura e stile globali
            let spacing = ui.spacing_mut();
            spacing.item_spacing = egui::Vec2::new(5.0, 7.0); // Spaziatura orizzontale e verticale
            spacing.text_edit_width = 300.0; // Larghezza del campo di testo

            ui.heading("Backup Configuration");

            ui.label("Source Path:");
            ui.horizontal(|ui| {
                // Campo per selezionare il percorso sorgente
                ui.text_edit_singleline(&mut self.source_path);

                // Pulsante per aprire il file dialog
                if ui.button("...").clicked() {
                    if let Some(path) = ConfigWindow::select_directory() {
                        self.source_path = path;
                    }
                }
            });

            ui.label("Destination Path:");
            ui.horizontal(|ui| {
                // Campo per selezionare il percorso destinazione
                ui.text_edit_singleline(&mut self.destination_path);

                // Pulsante per aprire il file dialog
                if ui.button("...").clicked() {
                    if let Some(path) = ConfigWindow::select_directory() {
                        self.destination_path = path;
                    }
                }
            });

            // ComboBox per scegliere il tipo di backup
            ui.label("Backup Type:");
            ComboBox::from_label("")
                .selected_text(&self.backup_type)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.backup_type, "full-disk".to_string(), "Full Disk");
                    ui.selectable_value(&mut self.backup_type, "directory".to_string(), "Directory");
                    ui.selectable_value(&mut self.backup_type, "selective".to_string(), "Selective");

                });

            // Mostra il campo "File Extensions" solo se il tipo di backup è "selective"
            if self.backup_type == "selective" {
                ui.label("File Extensions (comma separated):");
                ui.text_edit_singleline(&mut self.extensions_to_backup);
            }

            // Controlla i campi e imposta il messaggio di errore se necessario
            if self.source_path.trim().is_empty() {
                error_message.push_str("Source path is required.\n");
            }
            if self.destination_path.trim().is_empty() {
                error_message.push_str("Destination path is required.\n");
            }
            if self.backup_type == "selective" && self.extensions_to_backup.trim().is_empty() {
                error_message.push_str("Extensions are required for selective backup.\n");
            }
            if self.source_path == self.destination_path {
                error_message.push_str("Source and destination paths cannot be the same.\n");
            }

            // Mostra il pulsante di salvataggio con un messaggio di errore, se necessario
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                // Colore del pulsante
                let save_button_color = if is_valid {
                    Color32::from_rgb(100, 250, 100) // Verde se valido
                } else {
                    Color32::from_rgb(200, 100, 100) // Rosso se non valido
                };

                // Mostra il pulsante e disabilitalo se non valido
                let save_button = ui.add_enabled(
                    is_valid,
                    egui::Button::new("Save and Exit").fill(save_button_color),
                );

                // Salva e chiudi solo se il pulsante è cliccato ed è valido
                if save_button.clicked() {
                    self.save_config(config_file_path);
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }

                // Mostra il messaggio di errore se ci sono campi non validi
                if !is_valid {
                    ui.add_space(10.0);
                    ui.label(RichText::new(error_message).color(Color32::from_rgb(255, 0, 0)));
                }
            });
        });
    }
}




// Funzione per avviare la GUI solo se `config.toml` non esiste
pub fn show_gui_if_needed() -> Result<(), eframe::Error> {
    println!("Verifica se il file di configurazione esiste...");

    let exe_path: PathBuf = PathBuf::from(env::current_exe().unwrap().parent().unwrap());
    let mut config_file_path;

    config_file_path = exe_path.parent().unwrap().join("Resources/");


    if !config_file_path.join("config.toml").exists() {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([350f32, 325f32]),
            ..Default::default()
        };
        eframe::run_native(
            "Backup Configuration",
            options,
            Box::new(|_cc| Ok(Box::new(ConfigWindow::default()))),
        )
    } else {
        println!("Il file config.toml esiste già, la GUI non verrà mostrata.");
        Ok(())
    }
}

#[derive(Default)]
struct BackupWindow{
    should_close: Arc<Mutex<bool>>
}



impl eframe::App for BackupWindow {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {

        // Aggiungiamo un pop-up al centro dello schermo
        CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        // Titolo del pop-up
                        ui.heading("Do you want to proceed with backup?");
                        ui.add_space(10.0);
                        // Legenda con le istruzioni
                        ui.label("1. Scorri verso destra per eseguire il backup").highlight();
                        ui.label("2. Scorri verso l'alto per annullare il backup").highlight();
                        ui.label("3. Scorri in diagonale nel lato opposto per riconfigurare il backup").highlight();
                    });
                });

    }
}

// Funzione per mostrare la finestra di backup come pop-up
pub fn show_backup_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_always_on_top()
            .with_inner_size([500f32, 250f32])
            .with_decorations(true)
            .with_drag_and_drop(true),
        centered: true,
        ..Default::default()
    };

    // Avvia l'interfaccia grafica con la finestra di backup
    eframe::run_native(
        "Backup Confirmation",
        options,
        Box::new(|_cc| Ok(Box::new(BackupWindow {should_close: Arc::new(Mutex::new(false))}))),
    )
}

pub fn close_backup_window(should_close: Arc<Mutex<bool> >) {
    let mut should_close = should_close.lock().unwrap();
    *should_close = true;
}

pub fn is_window_open(should_close: Arc<Mutex<bool>>) -> bool {
    let should_close = should_close.lock().unwrap();
    !*should_close
}