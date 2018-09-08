use super::super::cart::{Cart, Mirroring};
use super::Mapper;


/// The simplest kind of mapper, with essentially no logic
pub struct Mapper0 {
    /// The cart this mapper handles the logic to
    cart: Cart,
    /// Should be either 0x2000 or 0x4000
    prg_size: u16
}

impl Mapper0 {
    pub fn new(cart: Cart) -> Self {
        // should be either 0x2000 or 0x4000
        let prg_size = cart.prg.len() as u16;
        Mapper0 { cart, prg_size }
    }
}

impl Mapper for Mapper0 {

    fn read(&self, address: u16) -> u8 {
        // addresses < 0x2000 are used for CHR rom
        match address {
            a if a < 0x2000 => self.cart.chr[a as usize],
            a if a >= 0x8000 => {
                let wrapped_addr = a % self.prg_size;
                self.cart.prg[wrapped_addr as usize]
            }
            a => {
                panic!("ABORT: Mapper0 unhandled read address: {:X}", a);
            }
        }
    }

    fn mirroring_mode(&self) -> Mirroring {
        self.cart.mirroring.clone()
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            a if a < 0x2000 => {
                self.cart.chr[a as usize] = value;
            }
            a if a >= 0x8000 => {
                let wrapped_adr = a % self.prg_size;
                self.cart.prg[wrapped_adr as usize] = value;
            }
            a => {
                panic!("ABORT: Mapper0 unhandled write address {:X}", a);
            }
        }
    }
}