use crate::apu::APU;
use crate::cart::Cart;
use crate::controller::ButtonState;
use crate::cpu::CPU;
use crate::memory::MemoryBus;
use crate::ports::{AudioDevice, VideoDevice};
use crate::ppu::PPU;

/// Used to act as an owner of everything needed to run a game
/// Is also responsible for holding ram,
/// as well as communication between processors.
pub struct Console {
    apu: APU,
    cpu: CPU,
    ppu: PPU,
}

impl Console {
    pub fn new(cart: Cart, sample_rate: u32) -> Self {
        let mut memory = MemoryBus::with_cart(cart);
        let ppu = PPU::new(&mut memory);
        let cpu = CPU::new(memory);
        Console {
            apu: APU::new(sample_rate),
            cpu,
            ppu,
        }
    }

    pub fn step<'a, A, V>(&'a mut self, audio: &mut A, video: &mut V) -> i32
    where
        A: AudioDevice,
        V: VideoDevice,
    {
        let cpucycles = self.cpu.step();
        let m = &mut self.cpu.mem;
        for _ in 0..cpucycles * 3 {
            self.ppu.step(m, video);
        }
        for _ in 0..cpucycles {
            self.apu.step(m, audio);
        }
        cpucycles
    }

    pub fn step_micros<'a, A, V>(&'a mut self, audio: &mut A, video: &mut V, micros: u32)
    where
        A: AudioDevice,
        V: VideoDevice,
    {
        // This emulates 1.79 cpu cycles per microsecond
        let mut cpu_cycles = ((micros * 179) / 100) as i32;
        while cpu_cycles > 0 {
            cpu_cycles -= self.step(audio, video);
        }
    }

    pub fn step_frame<'a, A, V>(&'a mut self, audio: &mut A, video: &mut V)
    where
        A: AudioDevice,
        V: VideoDevice,
    {
        self.step_micros(audio, video, 1_000_000 / 60);
    }

    pub fn update_controller(&mut self, buttons: ButtonState) {
        self.cpu.set_buttons(buttons);
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
