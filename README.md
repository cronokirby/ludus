# Ludus
An NES emulator written in Rust.

This was mostly started as a learning project, but the endgoal is
a usable emulator, not necessarily the most accurate.

## Installing
Given the few dependencies this app relies on, it should be as simple as:
```
git clone https://github.com/cronokirby/ludus
cd ludus
cargo run --release
```
(Note: you probably want to run on release if you don't want choppy
gameplay.)

## Controls
Right now, only hardwired mapping between keys and controllers is
supported, but in the future I plan to add remappable configuration via
some kind of file format.

| Button | Key |
| :----: | :-: |
| A      | J   |
| B      | K   |
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