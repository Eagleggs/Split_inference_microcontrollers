use std::cmp::{max, min};
use algo::{Layer, LayerWrapper};
use std::collections::HashMap;

pub fn quantize_layers_weights(layers: HashMap<i32, Box< dyn Layer>>) {
    let mut weights_min :f32 = 100000.;
    let mut weights_max: f32 = -100000.;
    //determine the float point range
    for i in 1..=layers.len(){
        let layer = layers.get(&(i as i32)).unwrap();
        let weights = layer.get_weights();
        weights.into_iter().for_each(|x|{
            weights_max = weights_max.max(x);
            weights_min = weights_min.min(x);
        });
    }
    //r = (q-z) * s; https://arxiv.org/abs/1712.05877v1
    let range = weights_max - weights_min;
    let scale = 255. / range; //u8
    let zero_point = -(weights_min / scale).round() as u8;
    for i in 0..layers.len(){
        let layer = layers.get(&(i as i32)).unwrap();
        let mut weights_quantized = layer.get_weights().into_iter().map(|x| ((x / scale).round() as u8 + zero_point) as u8 ).collect::<Vec<u8>>();

    }
    println!("{} {}",weights_max,weights_min);
    // todo!("quantize f32 to i8")
}
