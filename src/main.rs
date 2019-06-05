extern crate argparse;
extern crate ludus;

use argparse::{ArgumentParser, Store};

extern crate cpal;
extern crate minifb;

use ludus::controller::ButtonState;
use minifb::{Key, Scale, Window, WindowOptions};

use ludus::cart;
use ludus::console;
use ludus::ports;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Instant;

struct WindowDevice(Window);

impl ports::VideoDevice for WindowDevice {
    fn blit_pixels(&mut self, pixels: &ports::PixelBuffer) {
        self.0
            .update_with_buffer(pixels.as_ref())
            .expect("Couldn't update video device");
    }
}

struct SenderDevice(Sender<f32>);

impl ports::AudioDevice for SenderDevice {
    fn push_sample(&mut self, sample: f32) {
        self.0.send(sample).expect("Couldn't update audio device");
    }
}

/// Matches a string to corresponding screen scaling sheme
/// Matches anything besides 1, 2, and 4 to FitScreen
fn get_scale(s: &str) -> Scale {
    match s {
        "1" => Scale::X1,
        "2" => Scale::X2,
        "4" => Scale::X4,
        _ => Scale::FitScreen,
    }
}

fn get_console(rom_name: &str, sample_rate: u32) -> console::Console {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = File::open(rom_name).expect("Couldn't open the ROM file");
    file.read_to_end(&mut buffer)
        .expect("Couldn't read ROM file");
    console::Console::new(&buffer, sample_rate).unwrap_or_else(|e| match e {
        cart::CartReadingError::UnknownMapper(n) => panic!("Unkown Mapper: {}", n),
        cart::CartReadingError::UnrecognisedFormat => panic!("ROM was in an unrecognised format"),
    })
}

/// Runs a rom file with GUI and all
fn run(rom_name: &str, scale: Scale) {
    let (tx, rx) = channel::<f32>();
    let (sample_rate, _audio) = spawn_audio_loop(rx);
    let mut console = get_console(rom_name, sample_rate);
    let mut opts = WindowOptions::default();
    opts.scale = scale;
    let window = Window::new("Ludus - ESC to exit", 256, 240, opts).expect("Couldn't make window");
    run_loop(
        &mut console,
        &mut WindowDevice(window),
        &mut SenderDevice(tx),
    );
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
            if let cpal::StreamData::Output { buffer } = data {
                if let cpal::UnknownTypeOutputBuffer::F32(mut output) = buffer {
                    for sample in output.chunks_mut(channels) {
                        let value = rx.recv().unwrap();
                        for out in sample.iter_mut() {
                            *out = value;
                        }
                    }
                }
            }
        })
    });
    (sample_rate, child)
}

fn run_loop(console: &mut console::Console, window: &mut WindowDevice, sender: &mut SenderDevice) {
    let mut old = Instant::now();
    while window.0.is_open() && !window.0.is_key_down(Key::Escape) {
        let now = Instant::now();
        let duration = now.duration_since(old);
        old = now;

        if window.0.is_key_down(Key::Enter) {
            console.reset();
        }
        let buttons = ButtonState {
            a: window.0.is_key_down(Key::K),
            b: window.0.is_key_down(Key::J),
            select: window.0.is_key_down(Key::G),
            start: window.0.is_key_down(Key::H),
            up: window.0.is_key_down(Key::W),
            down: window.0.is_key_down(Key::S),
            left: window.0.is_key_down(Key::A),
            right: window.0.is_key_down(Key::D),
        };
        console.update_controller(buttons);
        console.step_micros(sender, window, duration.subsec_micros());
        window.0.update();
    }
}

fn main() {
    let mut rom_name = "".to_string();
    let mut scale = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut rom_name)
            .add_argument(&"ROM name", Store, "Path of ROM to use")
            .required();
        ap.refer(&mut scale)
            .add_option(&["--scale"], Store, "Screen scaling");
        ap.parse_args_or_exit();
    }
    println!("Using {} as ROM file", rom_name);
    run(&rom_name, get_scale(&scale));
}
