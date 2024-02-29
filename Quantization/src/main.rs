extern crate core;

use std::fs::File;
use algo::decode;
use crate::merge::merge_batchnorm;

mod merge;
mod quant;

pub fn main(){
    let file = File::open(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Algorithms\json_files\141.json").expect("Failed to open file");
    let original_layers = decode::decode_json(file);
    merge_batchnorm(original_layers);
}