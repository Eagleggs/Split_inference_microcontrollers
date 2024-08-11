extern crate core;
use algo::decode;
use quant::quant::{quantize_layers_activation, quantize_layers_weights};
use std::fs::File;
use quant::merge::merge_batchnorm;

pub fn main() {
    let file = File::open(r"..\Fused\fused_layers_141.json").expect("Failed to open file");
    let original_layers = decode::decode_json(file);
    // quantization of weights
    let (res, scale, zero) = quantize_layers_weights(&original_layers);
    // quantization of activations
    quantize_layers_activation(
        original_layers,
        r"..\Algorithms\images\test"
            .to_string(),
    );

    //layer fusion
//     let file = File::open(r"pc_code/Algorithms/json_files/141.json").expect("Failed to open file");
//     let original_layers = decode::decode_json(file);
//     merge_batchnorm(original_layers);
    //
}
