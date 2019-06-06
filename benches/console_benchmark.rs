#[macro_use]
extern crate criterion;
extern crate ludus;

use criterion::Criterion;
use criterion::black_box;
use ludus::*;

#[derive(Clone, Copy)]
pub struct NullDevice;

impl AudioDevice for NullDevice {
    fn push_sample(&mut self, _sample: f32) {
    }
}

impl VideoDevice for NullDevice {
    fn blit_pixels(&mut self, _pixels: &PixelBuffer) {
    }
}

fn step_frame(console: &mut Console) {
    console.step_frame(&mut NullDevice, &mut NullDevice);
}

fn criterion_benchmark(c: &mut Criterion) {
    let rom_bytes = include_bytes!("../test_roms/palette.nes");
    c.bench_function("console palette", move |b| {
        let cart = Cart::from_bytes(rom_bytes).unwrap();
        let mut console = Console::new(cart, 44000);
        b.iter(|| step_frame(black_box(&mut console)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);