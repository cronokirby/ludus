use super::cart::CartReadingError;
use super::apu::APU;
use super::cpu::CPU;
use super::memory::MemoryBus;
use super::ppu::PPU;

use super::minifb::Window;

use std::sync::mpsc::Sender;


/// Used to act as an owner of everything needed to run a game
/// Is also responsible for holding ram,
/// as well as communication between processors.
pub struct Console {
    cpu: CPU,
    ppu: PPU
}

impl Console {
    pub fn new(rom_buffer: &[u8], tx: Sender<f32>, sample_rate: u32)
        -> Result<Self, CartReadingError>
        {
        // Todo, use an actual sample rate
        let apu = APU::new(tx, sample_rate);
        // Will fail if the cart couldn't be read
        let mem_res = MemoryBus::with_rom(rom_buffer, apu);
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

    pub fn update_controller(&mut self,
        a: bool, b: bool, select: bool, start: bool,
        up: bool, down: bool, left: bool, right: bool)
    {
        self.cpu.set_buttons([a, b, select, start, up, down, left, right]);
    }

    /// Resets everything to it's initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.cpu.mem.reset();
        self.ppu.reset(&mut self.cpu.mem);
        self.ppu.clear_vbuffers();
    }

    pub fn print_cpu(&self) {
        self.cpu.print_state();
    }

    pub fn print_ram(&mut self, address: u16) {
        let read = self.cpu.read(address);
        println!("${:X} = {:X}", address, read)
    }
}