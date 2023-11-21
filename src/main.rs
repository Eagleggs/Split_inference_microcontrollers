use crate::lib::Layer;
use std::fs::File;
use std::io::Read;

mod convolution;
mod decode;
mod lib;

pub fn main() {
    let mut file = File::open("json_files/test2.json").expect("Failed to open file");
    let result = decode::decode_json(file);
    // Iterate over the entries and print each key-value pair
    let mut sorted = result.into_iter().collect::<Vec<(i16, Box<dyn Layer>)>>();
    sorted.sort_by_key(|&(x, _)| x);
    for (key, value) in sorted.into_iter() {
        println!("Layer: {}", key);
        // Assuming Layer has a debug implementation
        println!("Type: {:?}", value.identify());
        println!("Info: {:?}", value.get_info());
        println!("---");
    }

    print!("!");
}
