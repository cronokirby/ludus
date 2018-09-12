# Ludus
An NES emulator written in Rust.

This was mostly started as a learning project, but the endgoal is
a usable emulator, not necessarily the most accurate.

## Installing
Given the few dependencies this app relies on, it should be as simple as:
```
git clone https://github.com/cronokirby/ludus
cd ludus
cargo build --release
```
(Note: you probably want to run on release if you don't want choppy
gameplay.)

## Usage
### Running a game
```
ludus rom
```

### Rudimentary debugging
There's also an interactive mode, where the command line is used
to advance frame by frame and read values, this is mainly used for
development, and will likely be removed from a final version.
```
ludus rom -b
```
```
ludus rom --debug
```

ATM ludus only supports .nes (INES) files, which are the most common
format for nes roms.

## Controls
Right now, only hardwired mapping between keys and controllers is
supported, but in the future I plan to add remappable configuration via
some kind of file format.

| Button | Key |
| :----: | :-: |
| A      | K   |
| B      | J   |
| Start  | H   |
| Select | G   |
| Up     | W   |
| Down   | S   |
| Left   | A   |
| Right  | D   |

In addition, Esc closes the window, and Enter resets the console.

## Current state

### Working
- CPU, and thus core gameplay
- PPU, and thus graphics
- Basic controls via hardwired key mapping
- Mappers 0 and 2, so common games like Super Mario Bros, and Donkey Kong

### Not Working
- APU, so no audio atm
- Save States of any kind
- Keeping SRAM files for each game
- More complex Mappers


## Resources
I relied heavily on this very nicely written open source emulator:
https://github.com/fogleman/nes. The PPU code is shamelessly based off
their work, so major props to them! :P

This page https://wiki.nesdev.com/w/index.php/NES_reference_guide was and
still is my bible as I work on this project; kudos to the many
people who've contributed in some way over the years.