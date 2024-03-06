use std::cmp::{max, min};
use algo::{calculations, InfoWrapper, Layer, LayerWrapper, util};
use std::collections::HashMap;
use std::fs;
use algo::util::{pre_processing, read_and_store_image};

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
pub fn quantize_layers_weights(layers: &HashMap<i32, Box< dyn Layer>>) -> (Vec<Vec<u8>>,Vec<f32>,Vec<u8>) {
    let mut res = Vec::new();
    let mut scales = Vec::new();
    let mut zero_points = Vec::new();
    //determine the float point range
    for i in 1..=layers.len(){
        let l = layers.get(&(i as i32));
        match l {
            None => { continue; }
            _ =>{}
        }
        let layer = l.unwrap();
        let weights = layer.get_weights();
        if weights.is_empty() {
            continue
        }
        let weights_max = weights.iter().max_by(|a,b| a.partial_cmp(b).unwrap()).unwrap();
        let weights_min = weights.iter().min_by(|a,b| a.partial_cmp(b).unwrap()).unwrap();
        let range = weights_max - weights_min;
        let scale =  range / 255.;
        let zero_point = -(weights_min / scale).round() as u8; // z = -r / s + q
        let mut weights_quantized = layer.get_weights().into_iter().map(|x| (((x / scale).round()) + (zero_point as f32)) as u8 ).collect::<Vec<u8>>();
        // if i == 1{
        //     for j in 0..weights_quantized.len() {
        //         println!("{:?},{:?}",layer.get_weights()[j],weights_quantized[j]);
        //     }
        //     println!("{:?},{:?},{:?}",weights_min,weights_max,zero_point);
        // }

        res.push(weights_quantized);
        scales.push(scale);
        zero_points.push(zero_point);
        // print some property of the weights
        // let mean = weights.iter().map(|&x| x as f64).sum::<f64>() / weights.len() as f64;
        // let squared_diff_sum: f64 = weights
        //     .iter()
        //     .map(|&x| (x as f64 - mean).powi(2))
        //     .sum();
        // let mut variance = squared_diff_sum / weights.len() as f64;
        // variance = variance.sqrt();
        // println!("mean:{},std:{},max{},min{},range{}",mean,variance,weights_max,weights_min,range);
    }
    (res,scales,zero_points)
}
pub fn quantize_layers_activation(layers: HashMap<i32,Box<dyn Layer>>,calibration_set:String,weights_scale : Vec<f32>,weights_zero: Vec<u8>)->(Vec<u32>,Vec<u8>){
    // M = S1 * S2 / S3;
    let mut m_scale: Vec<u32> = vec![0;100];
    let mut scales : Vec<f32> = vec![0.;100];
    let mut zero_points : Vec<f32> = vec![0.;100];
    let mut residual_scale : Vec<f32> = vec![0.;100];
    let mut residual_zero_points : Vec<f32> = vec![0.;100];
    let residual_connections = vec![
        vec![10, 15], //10,15
        vec![20, 25], //20,25
        vec![25, 30], //25,30,
        vec![35, 40], //35,40
        vec![40, 45], //40,45
        vec![45, 50], //45,50
        vec![55, 60], //55,60
        vec![60, 65], //60,65
        vec![70, 75], //70,75
        vec![75, 80], //75,80
    ];
    // Read the directory entries
    if let Ok(entries) = fs::read_dir(calibration_set) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                // Check if it's a file (not a directory, symlink, etc.)
                if file_path.is_file() {
                    // Do something with the file, e.g., print its path
                    println!("File: {:?}", file_path);
                    println!("scales:{:?}",scales);
                    println!("zero_points:{:?}",zero_points);
                    let  image = read_and_store_image(file_path.to_str().unwrap()).unwrap();
                    let mut input = pre_processing(image);
                    let mut intermediate_output: Vec<Vec<Vec<f32>>> = Vec::new();
                    for i in 1..=layers.len() {
                        //find the maximum and minimum element in the input
                        let (mi, ma) = input.iter().flat_map(|row| row.iter().flat_map(|col| col.iter()))
                            .fold((f32::INFINITY, f32::NEG_INFINITY), |(mi, ma), &value| (mi.min(value), ma.max(value)));
                        //calculate the scale the zero point
                        let range = ma - mi;
                        let scale =  range / 255.;
                        let zero_point = -(mi / scale).round(); // z = -r / s + q
                        //use EWMA to get the scale and zero point
                        scales[i] = scales[i] * 0.9 + 0.1 * (scale);
                        zero_points[i] =  zero_points[i] * 0.9 + 0.1 * (zero_point);
                        //perform forward operation
                        if i == 88{
                            for i in 0..input.len(){
                                let temp = &input[i];
                                let mut acc = 0.;
                                temp.into_iter().for_each(|x| acc += x.into_iter().sum::<f32>());
                                let mean = acc / input[0].len() as f32 / input[0][0].len() as f32;
                                input[i] = vec![vec![mean]];
                            }//adaptive pooling
                            // continue
                        }
                        let layer = layers.get(&(i as i32)).unwrap();
                        let output_shape = layer.get_output_shape();
                        match layer.identify() {
                            "Convolution" => {
                                let mut output = vec![
                                vec![vec![0.; output_shape[2] as usize]; output_shape[1] as usize];
                                output_shape[0] as usize
                                ];
                                let mut flag = true;
                                for j in 0..output_shape[0] as usize {
                                    flag = true;
                                    let mut weights: Vec<f32> = Vec::new();
                                    let mut bias = layer.get_bias(j as i32);
                                    for k in 0..output_shape[1] as usize {
                                        for m in 0..output_shape[2] as usize {
                                            let pos = vec![j as i32, k as i32, m as i32];
                                            let inputs_p = layer.get_input(pos);
                                            //each output channel only need to sample weight once
                                            if flag {
                                                weights =
                                                    layer.get_weights_from_input(inputs_p.clone(), j as i32);
                                                flag = false;
                                            }
                                            let inputs =
                                                util::sample_input_from_p_zero_padding(inputs_p, &input);
                                            let result =
                                                calculations::vector_mul_b(inputs, weights.clone(), bias);
                                            output[j][k][m] = result;
                                        }
                                    }
                                }
                                //next layer's input = this layer's output
                                input = output;
                            }
                            "Batchnorm2d" => {
                                let Ok(_a) = layer.functional_forward(&mut input) else {
                                    panic!("wrong layer")
                                };
                            }
                            "Relu6" => {
                                let Ok(_a) = layer.functional_forward(&mut input) else {
                                    panic!("wrong layer")
                                };
                            }
                            "Linear" =>{
                                //todo!
                            }
                            _ => {}
                        }
                        //handle residual connection
                        for r in 0..residual_connections.len() {
                            if residual_connections[r][1] == i {
                            for j in 0..output_shape[0] as usize {
                                for k in 0..output_shape[1] as usize {
                                    for m in 0..output_shape[2] as usize {
                                        input[j][k][m] += intermediate_output[j][k][m];
                                        let (mi, ma) = input.iter().flat_map(|row| row.iter().flat_map(|col| col.iter()))
                                            .fold((f32::INFINITY, f32::NEG_INFINITY), |(mi, ma), &value| (mi.min(value), ma.max(value)));
                                        //calculate the scale the zero point
                                        let range = ma - mi;
                                        let scale =  range / 255.;
                                        let zero_point = -(mi / scale).round(); // z = -r / s + q
                                        //use EWMA to get the scale and zero point
                                        residual_scale[i] = residual_scale[i] * 0.9 + 0.1 * (scale);
                                        residual_zero_points[i] =  residual_zero_points[i] * 0.9 + 0.1 * (zero_point);
                                    }
                                }
                            }
                        }
                            if residual_connections[r][0] == i {
                                intermediate_output = input.clone();
                            }

                        }
                    }
                }
            }
        }
    } else {
        println!("Error reading directory");
    }
    //todo! read from calibration set, do forward propagation, find the min and max of each input and output,calculate the zero point and scale(residual connection counts as extra layer)
    (m_scale,zero_points.into_iter().map(|x| x.round() as u8).collect())

}
