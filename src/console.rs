use super::cart::CartReadingError;
use super::cpu::CPU;
use super::memory::MemoryBus;
use super::ppu::PPU;

use super::minifb::Window;


/// Used to act as an owner of everything needed to run a game
/// Is also responsible for holding ram,
/// as well as communication between processors.
pub struct Console {
    cpu: CPU,
    ppu: PPU
}

impl Console {
    pub fn new(rom_buffer: &[u8]) -> Result<Self, CartReadingError> {
        // Will fail if the cart couldn't be read
        let mem_res = MemoryBus::with_rom(rom_buffer);
        mem_res.map(|mut memory| {
            let ppu = PPU::new(&mut memory);
            let cpu = CPU::new(memory);
            Console { cpu, ppu }
        })
    }

    pub fn step(&mut self) -> i32 {
        let cpucycles = self.cpu.step();
        let m = &mut self.cpu.mem;
        for _ in 0..cpucycles * 3 {
            self.ppu.step(m);
        }
        cpucycles
    }

    pub fn step_micros(&mut self, micros: u32) {
        // This emulates 1.79 cpu cycles per microsecond
        let mut cpu_cycles = ((micros * 179) / 100) as i32;
        while cpu_cycles > 0 {
            cpu_cycles = cpu_cycles - self.step();
        }
    }

    pub fn step_frame(&mut self) {
        self.step_micros(1_000_000 / 60);
    }
    pub fn update_window(&self, window: &mut Window) {
        self.ppu.update_window(window);
    }

    /// Steps the console forward printing debug information
    pub fn debug_step(&mut self) {
        self.cpu.print_current_op();
        print!(" -> ");
        let cycles = self.cpu.step();
        for _ in 0..cycles * 3 {
            self.ppu.step(&mut self.cpu.mem);
        }
        self.cpu.print_state();
    }
}