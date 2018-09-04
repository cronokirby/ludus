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