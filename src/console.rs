use super::cart::CartReadingError;
use super::cpu::CPU;
use super::memory::MemoryBus;

use std::cell::RefCell;
use std::rc::Rc;


/// Used to act as an owner of everything needed to run a game
/// Is also responsible for holding ram,
/// as well as communication between processors.
pub struct Console {
    cpu: CPU
}

impl Console {
    pub fn new(rom_buffer: &[u8]) -> Result<Self, CartReadingError> {
        // Will fail if the cart couldn't be read
        let mem_res = MemoryBus::with_rom(rom_buffer);
        mem_res.map(|memory| {
            // this is done now because we need ram to be available
            let cpu = CPU::zeroed(Rc::new(RefCell::new(memory)));
            Console { cpu }
        })
    }
}