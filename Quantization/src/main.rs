extern crate core;

use crate::merge::merge_batchnorm;
use algo::decode;
use std::fs::File;
use crate::quant::quantize_layers_weights;

mod merge;
mod quant;

pub fn main() {
    let file = File::open(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Fused\fused_layers_141.json").expect("Failed to open file");
    let original_layers = decode::decode_json(file);
    quantize_layers_weights(original_layers);
    // merge_batchnorm(original_layers);
}
