use std::cmp::{max, min};
use algo::{InfoWrapper, Layer, LayerWrapper};
use std::collections::HashMap;
pub struct QuantizedWeightUnit {
    pub data: Vec<u8>,
    pub bias: u32,
    pub which_kernel: u16,
    pub count: i32,
    pub start_pos_in: Vec<i32>,
    pub info: InfoWrapper,
    pub m : u16,
    pub zero_points : (u8,u8,u8),
}
pub struct QuantizedMapping {
    pub count: Vec<u32>,
    pub map: Vec<Vec<u8>>,            // from which node,to which node
    // pub channel: Vec<u16>,            //used for batch norm,deleted after fusion with convolution,24/2/29
    pub padding_pos: Vec<Vec<u32>>,   //padding counts, when reached, should give 0
    pub end_pos: Vec<(u16, u8, u32)>, //phase,next_mcu,count
    pub zero_point : u8,
}
//r = (q-z) * s; https://arxiv.org/abs/1712.05877v1
pub fn quantize_layers_weights(layers: HashMap<i32, Box< dyn Layer>>) -> (Vec<Vec<u8>>,Vec<f32>,Vec<u8>) {
    let mut res = Vec::new();
    let mut scales = Vec::new();
    let mut zero_points = Vec::new();
    //determine the float point range
    for i in 0..layers.len(){
        let layer = layers.get(&(i as i32)).unwrap();
        let weights = layer.get_weights();
        let weights_max = weights.iter().max_by(|a,b| a.partial_cmp(b).unwrap()).unwrap();
        let weights_min = weights.iter().min_by(|a,b| a.partial_cmp(b).unwrap()).unwrap();
        let range = weights_max - weights_min;
        let scale = 255. / range;
        let zero_point = -(weights_min / scale).round() as u8; // z = -r / s + q
        let mut weights_quantized = layer.get_weights().into_iter().map(|x| ((x / scale).round() as u8 + zero_point) as u8 ).collect::<Vec<u8>>();
        res.push(weights_quantized);
        scales.push(scale);
        zero_points.push(zero_point);
    }
    (res,scales,zero_points)
}
pub fn quantize_layers_activation(layers: HashMap<i32,Box<dyn Layer>>,calibration_set:String,weights_scale : Vec<f32>,weights_zero: Vec<u8>)->(Vec<u32>,Vec<u8>){
    // M = S1 * S2 / S3;
    let mut m_scale: Vec<u32> = vec![0;layers.len()];
    let mut zero_points : Vec<u8> = vec![0;layers.len()];

    (m_scale,zero_points)

}
