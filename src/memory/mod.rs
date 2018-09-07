mod mapper0;

use super::cart::{Cart, CartReadingError, Mirroring};
use super::ppu::PPUState;
use self::mapper0::Mapper0;


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
            0 => Ok(Box::new(Mapper0::new(cart))),
            m => Err(CartReadingError::UnknownMapper(m))
        }
    }
}


/// Holds cart memory
pub struct MemoryBus {
    // Contains the mapper logic for interfacing with the cart
    // Each mapper has a different structure depending on what it
    // might need to keep track of, so we need to use dynamic dispatch.
    pub mapper: Box<Mapper>,
    pub ppustate: PPUState,
    ram: [u8; 0x2000]
}

impl MemoryBus {
    /// Creates a memory bus from a Cartridge buffer.
    /// Returns an error if the mapper is unkown or the cart fails to read
    pub fn with_rom(buffer: &[u8]) -> Result<Self, CartReadingError> {
        let cart_res = Cart::from_bytes(buffer);
        cart_res.and_then(|cart| Mapper::with_cart(cart).map(|mapper| {
            MemoryBus {
                mapper,
                ppustate: PPUState::new(),
                ram: [0; 0x2000]
            }
        }))
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            a if a < 0x2000 => self.ram[(a % 0x800) as usize],
            a if a < 0x4000 => {
                let adr = 0x2000 + a % 8;
                self.ppustate.read_register(&self.mapper, adr)
            }
            0x4016 => {
                panic!("Unimplemented Controller1 Read");
            }
            0x4017 => {
                panic!("Unimplemented Controller2 Read");
            }
            a if a >= 0x6000 => {
                self.mapper.read(address)
            }
            a => {
                panic!("Unhandled CPU read at {:X}", a);
            }
        }
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        match address {
            a if a < 0x2000 => self.ram[(a % 0x800) as usize] = value,
            a if a < 0x4016 => {
                panic!("Unimplemented PPU write at {:X}", a);
            }
            0x4016 => {
                panic!("Unimplemented Controller write");
            }
            0x4017 => {
                panic!("Unimplemented APU write");
            }
            a if a >= 0x6000 => self.mapper.write(address, value),
            a => {
                panic!("Unhandled CPU write at {:X}", a);
            }
        }
    }
}