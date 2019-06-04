mod mapper2;

use super::apu::APUState;
use super::cart::{Cart, CartReadingError, Mirroring};
use super::controller::Controller;
use super::cpu::CPUState;
use super::ppu::PPUState;

/// Used to abstract over the different types of Mappers
pub trait Mapper {
    fn read(&self, address: u16) -> u8;
    fn mirroring_mode(&self) -> Mirroring;
    fn write(&mut self, address: u16, value: u8);
}

impl Mapper {
    /// Dynamically assigns the correct mapper based on the cart.
    /// Returns an error if the mapper is unkown
    pub fn with_cart(cart: Cart) -> Result<Box<Mapper>, CartReadingError> {
        match cart.mapper {
            0 => Ok(Box::new(mapper2::Mapper2::new(cart))),
            2 => Ok(Box::new(mapper2::Mapper2::new(cart))),
            m => Err(CartReadingError::UnknownMapper(m)),
        }
    }
}

/// Holds cart memory
pub struct MemoryBus {
    // Contains the mapper logic for interfacing with the cart
    // Each mapper has a different structure depending on what it
    // might need to keep track of, so we need to use dynamic dispatch.
    pub mapper: Box<Mapper>,
    pub apu: APUState,
    pub cpu: CPUState,
    pub ppu: PPUState,
    // public for access by the cpu
    pub controller1: Controller,
    controller2: Controller,
    ram: [u8; 0x2000],
}

impl MemoryBus {
    /// Creates a memory bus from a Cartridge buffer.
    /// Returns an error if the mapper is unkown or the cart fails to read
    /// The apu needs to be passed explicitly, because the mem bus doesn't
    /// know things like the samplerate.
    pub fn with_rom(buffer: &[u8]) -> Result<Self, CartReadingError> {
        let cart_res = Cart::from_bytes(buffer);
        cart_res.and_then(|cart| {
            Mapper::with_cart(cart).map(|mapper| MemoryBus {
                mapper,
                apu: APUState::new(),
                cpu: CPUState::new(),
                ppu: PPUState::new(),
                controller1: Controller::new(),
                controller2: Controller::new(),
                ram: [0; 0x2000],
            })
        })
    }

    /// Clears ram as well as cpu and ppu state
    pub fn reset(&mut self) {
        self.ram = [0; 0x2000];
        self.cpu = CPUState::new();
        self.ppu = PPUState::new();
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            a if a < 0x2000 => self.ram[(a % 0x800) as usize],
            a if a < 0x4000 => {
                let adr = 0x2000 + a % 8;
                self.ppu.read_register(&*self.mapper, adr)
            }
            0x4014 => self.ppu.read_register(&*self.mapper, 0x4014),
            0x4015 => self.apu.read_register(address),
            0x4016 => self.controller1.read(),
            0x4017 => self.controller2.read(),
            a if a >= 0x6000 => self.mapper.read(address),
            a => {
                panic!("Unhandled CPU read at {:X}", a);
            }
        }
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            a if a < 0x2000 => self.ram[(a % 0x800) as usize] = value,
            a if a < 0x4000 => {
                let adr = 0x2000 + a % 8;
                self.ppu.write_register(&mut *self.mapper, adr, value);
            }
            a if a < 0x4014 => self.apu.write_register(a, value),
            0x4014 => {
                self.ppu.write_register(&mut *self.mapper, 0x4014, value);
                self.write_dma(value);
            }
            0x4015 => self.apu.write_register(address, value),
            0x4016 => {
                self.controller1.write(value);
                self.controller2.write(value);
            }
            0x4017 => self.apu.write_register(address, value),
            a if a >= 0x6000 => self.mapper.write(address, value),
            a => {
                panic!("Unhandled CPU write at {:X}", a);
            }
        }
    }

    fn write_dma(&mut self, value: u8) {
        let mut address = u16::from(value) << 8;
        // Stall for DMA
        self.cpu.add_stall(513);
        for _ in 0..256 {
            let oam_address = self.ppu.oam_address as usize;
            self.ppu.oam.0[oam_address] = self.cpu_read(address);
            self.ppu.oam_address = self.ppu.oam_address.wrapping_add(1);
            address += 1;
        }
    }
}
