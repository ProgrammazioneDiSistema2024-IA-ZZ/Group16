use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink};

pub fn play_sound(number: i32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file;
    // Usa un file .wav o un altro file audio
    if number == 0 {
        file = File::open("assets/blip-131856.mp3").unwrap(); // Assicurati di avere un file chiamato "beep.wav" nella cartella corrente
    }else if number == 1 {
        file = File::open("assets/success-48018.mp3").unwrap(); // Assicurati di avere un file chiamato "beep.wav" nella cartella corrente
    } else {
        file = File::open("assets/stop-13692.mp3").unwrap(); // Assicurati di avere un file chiamato "beep.wav" nella cartella corrente
    }
    let source = Decoder::new(BufReader::new(file)).unwrap();
    sink.append(source);

    sink.sleep_until_end(); // Attende la fine del suono
}