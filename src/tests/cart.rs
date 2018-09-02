use super::super::cart::*;

// Makes an ines file with anything filling the PRG and CHR
// 0xFF is used as a marker file for the beginning of PRG and CHR
fn make_ines(
    mirroring: Mirroring,
    has_battery: bool,
    trainer: bool,
    mapper: u8,
    prg_chunks: usize,
    chr_chunks: usize,
    ) -> Vec<u8>
{
    let trainer_offset = if trainer { 512 } else { 0 };
    let mut buffer = {
        let rom_size = 0x4000 * prg_chunks + 0x2000 * chr_chunks;
        Vec::with_capacity(16 + trainer_offset + rom_size)
    };
    let mut flag6 = 0;
    if mirroring.is_vertical() {
        flag6 |= 0b1;
    }
    if has_battery {
        flag6 |= 0b10;
    }
    if trainer {
        flag6 |= 0b100;
    }
    flag6 |= (mapper & 0x0F) << 4;
    let flag7 = (mapper & 0xF0) << 4;
    buffer.push(0x4E);
    buffer.push(0x45);
    buffer.push(0x53);
    buffer.push(0x1A);
    buffer.push(prg_chunks as u8);
    buffer.push(chr_chunks as u8);
    buffer.push(flag6);
    buffer.push(flag7);
    for _ in 8..16 {
        buffer.push(0);
    }
    for _ in 0..trainer_offset {
        buffer.push(0x1);
    }
    buffer.push(0xFF);
    for _ in 1..prg_chunks * 0x4000 {
        buffer.push(0x2);
    }
    buffer.push(0xFF);
    for _ in 1..chr_chunks * 0x2000 {
        buffer.push(0x3);
    }
    buffer
}

#[test]
fn cart_decoding() {
    let buffer = make_ines(
        Mirroring::Horizontal,
        true, false, 1, 1, 1);
    let cart_res = Cart::from_bytes(&buffer);
    assert!(cart_res.is_ok());
    let cart = cart_res.unwrap(); // we just asserted, so it's ok
    assert_eq!(cart.prg[0], 0xFF);
    assert_eq!(cart.chr[0], 0xFF);
    assert_eq!(cart.mapper, 1);
    assert!(!cart.mirroring.is_vertical());
    assert_eq!(cart.has_battery, true);
}