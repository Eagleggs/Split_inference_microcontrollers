use std::collections::HashMap;
use std::slice::SliceIndex;
use algo::Layer;

pub fn merge_batchnorm(layers:HashMap<i32,Box<dyn Layer>>){
    for i in 1..=layers.len(){
        let layer = layers.get(i.into()).unwrap();
        if layer.identify() == "Convolution" {
            let next_layer = layers.get((i + 1).into()).unwrap();
            if next_layer.identify() == "Batchnorm2d" {
                let conv_weights = layer.get_weights();
                let batch_norm = next_layer.get_weights();
            }
        }
    }
}