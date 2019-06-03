extern crate cpal;
extern crate minifb;

pub mod apu;
pub mod cart;
pub mod console;
pub mod controller;
pub mod cpu;
pub mod memory;
pub mod ppu;

#[cfg(test)]
mod tests;

use self::minifb::{Key, Scale, Window, WindowOptions};

use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

/// Matches a string to corresponding screen scaling sheme
/// Matches anything besides 1, 2, and 4 to FitScreen
pub fn get_scale(s: &str) -> Scale {
    match s {
        "1" => Scale::X1,
        "2" => Scale::X2,
        "4" => Scale::X4,
        _ => Scale::FitScreen,
    }
}

fn get_console(rom_name: &str, tx: Sender<f32>, sample_rate: u32) -> console::Console {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = File::open(rom_name).expect("Couldn't open the ROM file");
    file.read_to_end(&mut buffer)
        .expect("Couldn't read ROM file");
    console::Console::new(&buffer, tx, sample_rate).unwrap_or_else(|e| match e {
        cart::CartReadingError::UnknownMapper(n) => panic!("Unkown Mapper: {}", n),
        cart::CartReadingError::UnrecognisedFormat => panic!("ROM was in an unrecognised format"),
    })
}

/// Runs a rom file with GUI and all
pub fn run(rom_name: &str, scale: Scale) {
    let (tx, rx) = channel::<f32>();
    let (sample_rate, _audio) = spawn_audio_loop(rx);
    let mut console = get_console(rom_name, tx, sample_rate);
    let mut opts = WindowOptions::default();
    opts.scale = scale;
    let mut window =
        Window::new("Ludus - ESC to exit", 256, 240, opts).expect("Couldn't make window");
    run_loop(&mut console, &mut window);
}

fn spawn_audio_loop(rx: Receiver<f32>) -> (u32, thread::JoinHandle<()>) {
    let device = cpal::default_output_device().expect("Failed to get default output device");
    let format = device
        .default_output_format()
        .expect("Failed to get default output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());
    let sample_rate = format.sample_rate.0;
    let child = thread::spawn(move || {
        let channels = format.channels as usize;
        event_loop.run(move |_, data| {
            match data {
                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    for sample in buffer.chunks_mut(channels) {
                        let value = rx.recv().unwrap();
                        for out in sample.iter_mut() {
                            // convert between 0,1 and -1 1
                            *out = 2.0 * value - 1.0;
                        }
                    }
                }
                _ => {}
            }
        })
    });
    (sample_rate, child)
}

fn run_loop(console: &mut console::Console, window: &mut Window) {
    let mut old = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let duration = now.duration_since(old);
        old = now;

        if window.is_key_down(Key::Enter) {
            console.reset();
        }

        console.update_controller(
            window.is_key_down(Key::K),
            window.is_key_down(Key::J),
            window.is_key_down(Key::G),
            window.is_key_down(Key::H),
            window.is_key_down(Key::W),
            window.is_key_down(Key::S),
            window.is_key_down(Key::A),
            window.is_key_down(Key::D),
        );
        console.step_micros(duration.subsec_micros());
        console.update_window(window);
    }
}
