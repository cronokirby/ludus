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

    /// Advance the console by a single CPU cycle.
    /// 
    /// This needs access to the audio and video devices, because the APU
    /// may generate audio samples, and the PPU may generate a frame.
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

    /// Advance the console by a certain number of micro seconds.
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

    /// Advance the console until the next frame.
    /// 
    /// Unlike the other step methods, this is not based on timing, but
    /// based on waiting until the ppu actually generates a video frame.
    /// This is more useful for applications that want to do something
    /// at the start of every frame, like playing the next frame of input
    /// from a recorded script, or things like that.
    pub fn step_frame<'a, A, V>(&'a mut self, audio: &mut A, video: &mut V)
    where
        A: AudioDevice,
        V: VideoDevice,
    {
        let mut frame_happened = false;
        while !frame_happened {
            let cpucycles = self.cpu.step();
            let m = &mut self.cpu.mem;
            for _ in 0..cpucycles * 3 {
                frame_happened = self.ppu.step(m, video) || frame_happened;
            }
            for _ in 0..cpucycles {
                self.apu.step(m, audio);
            }
        }
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
}
