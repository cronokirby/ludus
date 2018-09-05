use super::cart::CartReadingError;
use super::cpu::CPU;
use super::memory::MemoryBus;


/// Used to act as an owner of everything needed to run a game
/// Is also responsible for holding ram,
/// as well as communication between processors.
pub struct Console {
    cpu: CPU,
    memory: MemoryBus,
    ram: [u8; 0x2000]
}

impl Console {
    pub fn new(rom_buffer: &[u8]) -> Result<Self, CartReadingError> {
        let cpu = CPU::zeroed();
        let ram = [0; 0x2000];
        // Will fail if the cart couldn't be read
        let mem_res = MemoryBus::with_rom(rom_buffer);
        mem_res.map(|memory| {
            let mut console = Console { cpu, memory, ram };
            // this is done now because we need ram to be available
            console.cpu.reset(&console.memory);
            console
        })
    }
}