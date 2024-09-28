mod display_window;

fn main(){
    if let Err(e) = display_window::show_gui_if_needed() {
        eprintln!("Errore nella generazione della GUI: {}", e);
    }
}