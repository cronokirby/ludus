use super::cart::CartReadingError;
use super::cpu::CPU;
use super::memory::MemoryBus;

use super::minifb::Window;

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

    pub fn step(&mut self) -> i32 {
        self.cpu.step()
    }

    pub fn step_micros(&mut self, micros: u32) {
        let mut cpu_cycles = ((micros * 179) / 100) as i32;
        while cpu_cycles > 0 {
            cpu_cycles = cpu_cycles - self.step();
        }
    }
    pub fn update_window(&self, window: &mut Window) {
        let buffer = [0x00FF00FF; 256 * 240];
        window.update_with_buffer(&buffer).unwrap();
    }

    /// Steps the console forward printing debug information
    pub fn debug_step(&mut self) {
        self.cpu.print_current_op();
        print!(" -> ");
        self.cpu.step();
        self.cpu.print_state();
    }
}