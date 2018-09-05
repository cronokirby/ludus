mod mapper0;

use super::cart::{Cart, CartReadingError};
use self::mapper0::Mapper0;


/// Used to abstract over the different types of Mappers
pub trait Mapper {
    fn read(&self, address: u16) -> u8;
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
    mapper: Box<Mapper>,
}

impl MemoryBus {
    /// Creates a memory bus from a Cartridge buffer.
    /// Returns an error if the mapper is unkown or the cart fails to read
    pub fn with_rom(buffer: &[u8]) -> Result<Self, CartReadingError> {
        let cart_res = Cart::from_bytes(buffer);
        cart_res.and_then(|cart| Mapper::with_cart(cart).map(|mapper| {
            MemoryBus { mapper }
        }))
    }
}