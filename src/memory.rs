use super::cart::Cart;


/// Used to abstract over the different types of Mappers
trait Mapper {
    // Empty for now, needs read / write methods
}


/// Holds cart memory
pub struct MemoryBus {
    /// Holds the cart where PRG and CHR ROM are located
    cart: Cart,
    /// Contains the mapper logic for interfacing with the cart
    // Each mapper has a different structure depending on what it
    // might need to keep track of, so we need to use dynamic dispatch.
    mapper: Box<Mapper>,
}