pub mod cart;
pub mod console;
pub mod cpu;
pub mod memory;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::Read;


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