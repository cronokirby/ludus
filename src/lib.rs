extern crate minifb;

pub mod cart;
pub mod console;
pub mod cpu;
pub mod memory;
pub mod ppu;

#[cfg(test)]
mod tests;

use self::minifb::{Key, Scale, WindowOptions, Window};

use std::fs::File;
use std::io::{Read, stdin};
use std::time::Instant;


/// Attempts to disassemble a rom, panicing on exits
pub fn disassemble(rom_name: &str) {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = File::open(rom_name)
        .expect("Couldn't open the ROM file");
    file.read_to_end(&mut buffer).expect("Couldn't read ROM file");
    let cart = cart::Cart::from_bytes(&buffer)
        .expect("Invalid ROM format");
    println!("Disassembling ROM...");
    cpu::disassemble(&cart.prg);
}

enum Interaction {
    Advance,
    Run
}

/// Gets an interaction by reading a line
/// Returns None if no valid Interaction could be fetched
fn get_interaction() -> Option<Interaction> {
    let mut input = String::new();
    match stdin().read_line(&mut input) {
        Err(_) => None,
        Ok(_) => {
            let s = input.trim();
            match s {
                "" => Some(Interaction::Advance),
                "run" => Some(Interaction::Run),
                _ => None
            }
        }
    }
}

fn get_console(rom_name: &str) -> console::Console {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = File::open(rom_name)
        .expect("Couldn't open the ROM file");
    file.read_to_end(&mut buffer).expect("Couldn't read ROM file");
    console::Console::new(&buffer).unwrap_or_else(|e| {
        match e {
            cart::CartReadingError::UnknownMapper(n) => {
                panic!("Unkown Mapper: {}", n)
            }
            cart::CartReadingError::UnrecognisedFormat => {
                panic!("ROM was in an unrecognised format")
            }
        }
    })
}

pub fn debug(rom_name: &str) {
    let mut console = get_console(rom_name);
    let mut run = false;
    loop {
        // just loop steps forever
        if run {
            console.debug_step();
            continue;
        }
        match get_interaction() {
            None => println!("Unknown command"),
            Some(Interaction::Advance) => {
                console.debug_step();
            }
            Some(Interaction::Run) => run = true
        }
    }
}


/// Runs a rom file with GUI and all
pub fn run(rom_name: &str) {
    let mut console = get_console(rom_name);
    let mut opts = WindowOptions::default();
    opts.scale = Scale::X4;
    let mut window = Window::new(
        "Test - ESC to exit", 256, 240, opts
    ).expect("Couldn't make window");

    let mut old = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let duration = now.duration_since(old);
        old = now;
        let new_micros = duration.subsec_micros();
        let enter_down = window.is_key_down(Key::Enter);
        console.step_micros(duration.subsec_micros());
        console.update_window(&mut window);
    }
}