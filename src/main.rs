extern crate argparse;
extern crate ludus;

use argparse::{ArgumentParser, Store};


fn main() {
    let mut rom_name = "".to_string();
    let mut scale = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut rom_name)
            .add_argument(&"ROM name", Store, "Path of ROM to use")
            .required();
        ap.refer(&mut scale)
            .add_option(&["--scale"], Store, "Screen scaling");
        ap.parse_args_or_exit();
    }
    println!("Using {} as ROM file", rom_name);
    ludus::run(&rom_name, ludus::get_scale(&scale));
}