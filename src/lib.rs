pub mod cart;
pub mod console;
pub mod cpu;
pub mod memory;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{Read, stdin};


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

pub fn debug(rom_name: &str) {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = File::open(rom_name)
        .expect("Couldn't open the ROM file");
    file.read_to_end(&mut buffer).expect("Couldn't read ROM file");
    let mut console = console::Console::new(&buffer).unwrap_or_else(|e| {
        match e {
            cart::CartReadingError::UnknownMapper(n) => {
                panic!("Unkown Mapper: {}", n)
            }
            cart::CartReadingError::UnrecognisedFormat => {
                panic!("ROM was in an unrecognised format")
            }
        }
    });
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