mod interface;
mod wad;

use wad::WadFile;

use crate::interface::Interface;

fn main() {
    let wad_file = WadFile::load_from("./doom1.wad");
    println!("# Subsectors: {:?}", wad_file.levels[0].subsectors.len());
    let mut interface = Interface::new();
    interface.run(&wad_file);
}
