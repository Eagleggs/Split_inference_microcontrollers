extern crate core;
use algo::decode;
use quant::quant::{quantize_layers_activation, quantize_layers_weights};
use std::fs::File;

pub fn main() {
    let file = File::open(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Fused\fused_layers_141.json").expect("Failed to open file");
    let original_layers = decode::decode_json(file);
    let (res, scale, zero) = quantize_layers_weights(&original_layers);
    quantize_layers_activation(
        original_layers,
        r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Algorithms\images\test"
            .to_string(),
    );
    // merge_batchnorm(original_layers);
}
