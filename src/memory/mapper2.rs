use super::super::cart::{Cart, Mirroring};
use super::Mapper;

pub struct Mapper2 {
    cart: Cart,
    prg_banks: i32,
    prgbank1: usize,
    prgbank2: usize,
}

impl Mapper2 {
    pub fn new(cart: Cart) -> Self {
        let prg_banks = cart.prg.len() / 0x4000;
        let prgbank1 = 0;
        let prgbank2 = prg_banks - 1;
        Mapper2 {
            cart,
            prg_banks: prg_banks as i32,
            prgbank1,
            prgbank2,
        }
    }
}

impl Mapper for Mapper2 {
    fn read(&self, address: u16) -> u8 {
        match address {
            a if a < 0x2000 => self.cart.chr[a as usize],
            a if a >= 0xC000 => {
                let shifted = (address - 0xC000) as usize;
                let index = self.prgbank2 * 0x4000 + shifted;
                self.cart.prg[index]
            }
            a if a >= 0x8000 => {
                let shifted = (address - 0x8000) as usize;
                let index = self.prgbank1 * 0x4000 + shifted;
                self.cart.prg[index]
            }
            a if a >= 0x6000 => {
                let shifted = (address - 0x6000) as usize;
                self.cart.sram[shifted]
            }
            a => {
                panic!("Mapper2 unhandled read at {:X}", a);
            }
        }
    }

    fn mirroring_mode(&self) -> Mirroring {
        self.cart.mirroring.clone()
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            a if a < 0x2000 => self.cart.chr[a as usize] = value,
            a if a >= 0x8000 => {
                let bank = i32::from(value) % self.prg_banks;
                self.prgbank1 = bank as usize;
            }
            a if a >= 0x6000 => {
                let shifted = (address - 0x6000) as usize;
                self.cart.sram[shifted] = value;
            }
            a => {
                panic!("Mapper2 unhandled write at {:X}", a);
            }
        }
    }
}
