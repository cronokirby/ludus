extern crate argparse;
extern crate ludus;

use argparse::{ArgumentParser, StoreTrue, Store};


fn main() {
    let mut rom_name = "".to_string();
    let mut disasm = false;
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut rom_name)
            .add_argument(&"ROM name", Store, "Path of ROM to use")
            .required();
        ap.refer(&mut disasm)
            .add_option(&["-d", "--disasm"], StoreTrue, "Disassemble ROM");
        ap.parse_args_or_exit();
    }
    println!("Using {} as ROM file", rom_name);
    if disasm {
        ludus::disassemble(&rom_name);
    }
}