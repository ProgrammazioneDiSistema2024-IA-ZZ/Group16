use eframe::egui::{self, CentralPanel};

#[derive(Default)]
pub struct MyApp {
    pub my_text: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Egui + eframe Example");
            ui.label("Questo è un campo di testo modificabile:");
            ui.text_edit_singleline(&mut self.my_text);
        });
    }
}

pub fn show_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Egui Example - Text Editable",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}
