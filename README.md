[![crates.io](https://img.shields.io/crates/v/ludus.svg)](https://crates.io/crates/ludus)
[![docs.rs](https://docs.rs/ludus/badge.svg)](https://docs.rs/ludus/)

# ludus

**Ludus** is a crate providing the core logic of an NES emulator. Unlike other
crates, **Ludus** is not a standalone application. Instead, **Ludus** is a crate
designed to be easily embedded in an application. For an example of using
**Ludus** to make a GUI emulator, see
[ludus-emu](https://github.com/cronokirby/ludus-emu).

The advantage of being headless is that **Ludus** is easily useable in contexts
outside of a standalone application. For example, this crate could be used
to train agents the play NES games, or to generate screenshots from NES games,
or to generate plots of RAM, etc. By being headless, **Ludus** can be used
in your own emulator application in whatever way you want.

## Features
- CPU emulation
- Video emulation
- Audio emulation
- Parsing rom data from `.ines` files.
- Mappers 0, 1, and 2, so many common games.

## Usage
Let's first import the main types used in **Ludus**:
```rust
use ludus::*;
```

The main emulator type is `Console`. Before we can create a `Console`, we need
a cartridge to play. We can create a `Cart` type by reading an `.ines` file.

```rust
let bytes: &[u8] = read_ines_bytes();
let cart = Cart::from_bytes(bytes).unwrap();
```

Creating a cartridge will naturally fail if the ROM data wasn't valid.

Once we have a cartridge, we can create a console to play this cartridge:
```rust
let console = Console::new(cart, sample_rate);
```

Creating a console requires a cartridge, as well as a sample rate for the audio
process unit (APU). Normally, if you're using some crate that allows you to play
audio to a device, you should have access to this sample_rate.

At any point in time we can reset the console like so:
```rust
console.reset();
```

We can also update the state of the buttons using the `ButtonState` struct:
```rust
let mut buttons = ButtonState::default();
buttons.a = true;
console.update_controller(buttons);
```

Now to actually start doing some emulation, we need to `step` the console forward.
Anytime we advance emulation however, the APU might generate audio samples, and
the PPU might generate video frames. To handle these, we need to provide a device
that can handle the audio samples, and a device to handle the video frames.

For handling audio, we have the `AudioDevice` trait:
```rust
trait AudioDevice {
    fn push_sample(&mut self, sample: f32)
}
```
A device implementing this trait should be able to receive an audio sample,
in the range `[-1, 1]` and to audio stuff with that information. The sample rate
passed to the console determines how often the APU will generate samples and push
them to this device.

For handling video, we have the `VideoDevice` trait:
```rust
trait VideoDevice {
    fn blit_pixels(&mut self, pixels: &PixelBuffer)
}
```
This device should be able to receive a frame of pixels, and display that on  
screen, or whatever else you might want to do with the video data.
The pixelbuffer contains 256x240 ARGB pixels, in row major format.

If you don't want to handle audio or video, you can simple create an empty struct
that does nothing for both traits:

```rust
#[derive(Clone, Copy)]
pub struct NullDevice;

impl AudioDevice for NullDevice {
    fn push_sample(&mut self, sample: f32) {
    }
}

impl VideoDevice for NullDevice {
    fn blit_pixels(&mut self, pixels: &PixelBuffer) {
    }
}
```

Now that we have the devices set up, we can start doing some emulation.

The simplest method to advance the console is `step`:
```rust
pub fn step<'a, A, V>(&'a mut self, audio: &mut A, video: &mut V) -> i32 where
    A: AudioDevice,
    V: VideoDevice,
```
This will advance the `Console` forward by one cpu cycle. This is only useful
if you want to be able to see things advance very very slowly. If you're
something automated, like a bot, you want to use `step_frame` instead, since
most games won't even look at input more than once per frame anyways.

The next method is `step_micros`:
```rust
pub fn step_micros<'a, A, V>(
    &'a mut self,
    audio: &mut A,
    video: &mut V,
    micros: u32
) where
    A: AudioDevice,
    V: VideoDevice, 
```

This method will instead advance the emulator by a certain number of microseconds.
This is the most useful method if you're implementing your own GUI and want
to advance the emulator in some kind of game loop.

An example of doing such a loop might look like this:
```rust
let mut old = Instant::now();
loop {
    let now = Instant::now();
    let duration = now.duration_since(old);
    old = now;
    console.step_micros(audio, video, duration.subsec_micros());
}
```

The final method allows you to advance the emulator by a full frame:
```rust
pub fn step_frame<'a, A, V>(&'a mut self, audio: &mut A, video: &mut V) where
    A: AudioDevice,
    V: VideoDevice,
```
This is useful if you're training a bot, because games will only look at input
once per frame. So you'd set input for that frame, then advance once frame, then
set input, etc. Note that this is based on *timing* and not on waiting for the PPU
to send a VBlank interrupt to the CPU, which tells the CPU that a frame has
occurred, and is used by most games to handle things like input. So if you've
advanced halfway through a frame, and then call this method, you'll end up
halfway through the next, because this method advances for the duration
of a frame, and not until the next frame occurrs.

## Resources

I relied heavily on this very nicely written open source emulator: https://github.com/fogleman/nes.

This page https://wiki.nesdev.com/w/index.php/NES_reference_guide was and still is my bible as I work on this project; kudos to the many people who've contributed in some way over the years.
