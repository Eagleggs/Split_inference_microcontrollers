use crate::lib::Layer;
use std::fs::File;
use std::io::Read;
mod convolution;
mod decode;
mod lib;
mod linear;

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
        value.print_weights_shape();
        println!("---");
    }

    print!("!");
}
#[cfg(test)]
mod tests{
    use crate::lib::Conv;
    use super::*;

    #[test]
    fn test_convolution(){
        let mut file = File::open("json_files/test_convolution.json").expect("Failed to open file");
        let result = decode::decode_json(file);
        let r = result.get(&1).expect("failed");
        let output_shape = r.get_output_shape();
        for i in 0..output_shape[0]{
            for j in 0..output_shape[1]{
                for m in 0..output_shape[2]{
                    let pos = vec!(i,j,m);
                    let inputs = r.get_input(pos);
                }
            }
        }
        println!("1");
    }
}