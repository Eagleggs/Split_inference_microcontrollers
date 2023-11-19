use std::fs::File;
use std::io::Read;

mod decode;

pub fn main(){
    let mut file = File::open("json_files/test.json").expect("Failed to open file");
    let result = decode::decode_json(file);
    // Access the parsed data as needed

    print!("!");
}