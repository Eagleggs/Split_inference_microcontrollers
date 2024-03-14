use algo::util::{pre_processing, read_and_store_image};
use algo::{calculations, util, InfoWrapper, Layer, LayerWrapper, WeightUnit, QuantizedWeightUnit, QuantizedMapping, Mapping};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};


//r = (q-z) * s; https://arxiv.org/abs/1712.05877v1
pub fn quantize_layers_weights(
    layers: &HashMap<i32, Box<dyn Layer>>,
) -> (Vec<Vec<u8>>, Vec<f32>, Vec<f32>) {
    let mut res = Vec::new();
    let mut scales = vec![0.;100];
    let mut zero_points = vec![0.;100];
    //determine the float point range
    for i in 1..=layers.len() {
        let l = layers.get(&(i as i32));
        match l {
            None => {
                continue;
            }
            _ => {}
        }
        let layer = l.unwrap();
        let weights = layer.get_weights();
        if weights.is_empty() {
            continue;
        }
        let weights_max = weights
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let weights_min = weights
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let range = weights_max - weights_min;
        let scale = range / 255.;
        let zero_point = -(weights_min / scale); // z = -r / s + q
        let mut weights_quantized = layer
            .get_weights()
            .into_iter()
            .map(|x| (((x / scale)) + (zero_point)).round() as u8)
            .collect::<Vec<u8>>();

        res.push(weights_quantized);
        scales[i] = scale;
        zero_points[i] = zero_point;
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
    // println!("scales:{:?},zero:{:?}",scales,zero_points);
    (res, scales, zero_points)
}
pub fn quantize_layers_activation(
    layers: HashMap<i32, Box<dyn Layer>>,
    calibration_set: String,
) -> (Vec<u32>, Vec<u8>) {
    // M = S1 * S2 / S3;
    let mut m_scale: Vec<u32> = vec![0; 100];
    let mut scales: Vec<f32> = vec![0.; 100];
    let mut zero_points: Vec<f32> = vec![0.; 100];
    let mut residual_scale: Vec<f32> = vec![0.; 100];
    let mut residual_zero_points: Vec<f32> = vec![0.; 100];
    let mut test_result = Vec::new();
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
                    println!("scales:{:?}", scales);
                    println!("zero_points:{:?}", zero_points);
                    println!("resi scales:{:?}", residual_scale);
                    println!("resi zero:{:?}", residual_zero_points);
                    let image = read_and_store_image(file_path.to_str().unwrap()).unwrap();
                    let mut input = pre_processing(image);
                    let mut intermediate_output: Vec<Vec<Vec<f32>>> = Vec::new();
                    for i in 1..=layers.len() + 1 {
                        //find the maximum and minimum element in the input
                        let (mi, ma) = input
                            .iter()
                            .flat_map(|row| row.iter().flat_map(|col| col.iter()))
                            .fold((f32::INFINITY, f32::NEG_INFINITY), |(mi, ma), &value| {
                                (mi.min(value), ma.max(value))
                            });
                        //calculate the scale the zero point
                        let range = ma - mi;
                        let scale = range / 255.;
                        let zero_point = -(mi / scale).round(); // z = -r / s + q
                        //use EWMA to get the scale and zero point
                        scales[i] = scales[i] * 0.99 + 0.01 * (scale);
                        zero_points[i] = zero_points[i] * 0.99 + 0.01 * (zero_point);
                        //perform forward operation
                        if i == 88 {
                            for i in 0..input.len() {
                                let temp = &input[i];
                                let mut acc = 0.;
                                temp.into_iter()
                                    .for_each(|x| acc += x.into_iter().sum::<f32>());
                                let mean = acc / input[i].len() as f32 / input[i][0].len() as f32;
                                input[i] = vec![vec![mean]];
                            } //adaptive pooling
                            // continue
                        }
                        if i > layers.len() {
                            // print!("!!!!!!!");
                            test_result = input;
                            break;
                        }
                        let layer = layers.get(&(i as i32)).unwrap();
                        let output_shape = layer.get_output_shape();
                        match layer.identify() {
                            "Convolution" => {
                                let mut output = vec![
                                    vec![
                                        vec![0.; output_shape[2] as usize];
                                        output_shape[1] as usize
                                    ];
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
                                                weights = layer.get_weights_from_input(
                                                    inputs_p.clone(),
                                                    j as i32,
                                                );
                                                flag = false;
                                            }
                                            let inputs = util::sample_input_from_p_zero_padding(
                                                inputs_p, &input,
                                            );
                                            let result = calculations::vector_mul_b(
                                                inputs,
                                                weights.clone(),
                                                bias,
                                            );
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
                            "Linear" => {
                                assert_eq!(input.len(), 1280);
                                assert!(input[0].len() == 1 && input[0][0].len() == 1);
                                let mut output = vec![vec![vec![0.0]]; 1000];
                                let weights = layer.get_weights();
                                if let InfoWrapper::Linear(info) = layer.get_info() {
                                    let weights_shape = [info.c_out, info.c_in]; //1000,1280
                                    for i in 0..weights_shape[0] as usize {
                                        let mut acc = 0.;
                                        for j in 0..weights_shape[1] as usize {
                                            acc += weights[i * weights_shape[1] as usize + j]
                                                * input[j][0][0];
                                        }
                                        output[i][0][0] = acc + layer.get_bias(i as i32);
                                    }
                                } else {
                                    panic!("not a linear layer")
                                }
                                input = output;
                            }
                            _ => {}
                        }
                        //handle residual connection
                        for r in 0..residual_connections.len() {
                            if residual_connections[r][1] == i {
                                let (mi, ma) = input
                                    .iter()
                                    .flat_map(|row| row.iter().flat_map(|col| col.iter()))
                                    .fold(
                                        (f32::INFINITY, f32::NEG_INFINITY),
                                        |(mi, ma), &value| (mi.min(value), ma.max(value)),
                                    );
                                //calculate the scale the zero point
                                let range = ma - mi;
                                let scale = range / 255.;
                                let zero_point = -(mi / scale).round(); // z = -r / s + q
                                //use EWMA to get the scale and zero point
                                residual_scale[i] = residual_scale[i] * 0.99 + 0.01 * (scale);
                                residual_zero_points[i] =
                                    residual_zero_points[i] * 0.99 + 0.01 * (zero_point);
                                for j in 0..output_shape[0] as usize {
                                    for k in 0..output_shape[1] as usize {
                                        for m in 0..output_shape[2] as usize {
                                            input[j][k][m] += intermediate_output[j][k][m];
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
    (
        m_scale,
        zero_points.into_iter().map(|x| x.round() as u8).collect(),
    )
}
pub fn calculate_quantization(original_weights: Vec<Vec<WeightUnit>>,original_mapping:Vec<Mapping>,weight_scales : Vec<f32>,weight_zero_points : Vec<f32>,layer_id:usize)->(Vec<Vec<QuantizedWeightUnit>>,Vec<QuantizedMapping>) {
    //pre calculated values using quantize_layers_activation function
    let scales:Vec<f32> = vec![0.0, 0.017818455, 0.017857317, 0.010181317, 0.07693508, 0.023514507, 0.041469865, 0.04173334, 0.019598939, 0.040599767, 0.016114194, 0.026925236, 0.01976499, 0.0057206596, 0.022290565, 0.011403129, 0.040337086, 0.020162629, 0.0084821, 0.022076631, 0.009201978, 0.021326661, 0.008397463, 0.0041799145, 0.015718713, 0.006351808, 0.026975844, 0.0077458634, 0.004276918, 0.013787641, 0.0064519304, 0.03300309, 0.012218302, 0.0061818487, 0.013625881, 0.007596627, 0.017282467, 0.0056892363, 0.0030426302, 0.010880587, 0.0047031413, 0.019367196, 0.005499944, 0.0027530068, 0.011284228, 0.0051408643, 0.020728296, 0.006147768, 0.002869781, 0.012964951, 0.00812551, 0.022537425, 0.0075556524, 0.004070429, 0.014004531, 0.007840167, 0.016347256, 0.008830495, 0.0041988418, 0.018410122, 0.00890296, 0.019542953, 0.010620198, 0.0057104784, 0.018760901, 0.008726812, 0.026837224, 0.010573086, 0.0054687196, 0.012701936, 0.008290729, 0.013501433, 0.00947222, 0.005058123, 0.018148044, 0.0073697055, 0.019419255, 0.008566049, 0.0048613106, 0.017253218, 0.0076185213, 0.0346372, 0.009283503, 0.0035971922, 0.010591191, 0.004404244, 0.0094860755, 0.069449075, 0.023512602, 0.09354488, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let zero_points : Vec<f32> = vec![0.0, 114.38545, 109.03897, 0.0, 108.3703, 0.0, 129.86067, 133.89502, 0.0, 153.77016, 0.0, 117.14856, 179.86311, 0.0, 123.245834, 0.0, 120.96629, 147.66423, 0.0, 147.94286, 0.0, 136.40172, 127.67802, 0.0, 150.21545, 0.0, 132.43462, 114.412674, 0.0, 135.89473, 0.0, 134.72113, 126.012344, 0.0, 112.941055, 0.0, 132.12167, 118.3496, 0.0, 144.18782, 0.0, 130.06107, 127.58742, 0.0, 137.86534, 0.0, 130.81909, 136.1324, 0.0, 103.3621, 0.0, 131.53062, 117.90426, 0.0, 112.85589, 0.0, 125.70349, 134.3532, 0.0, 127.507, 0.0, 126.38503, 123.73302, 0.0, 136.63579, 0.0, 124.32738, 126.83683, 0.0, 91.00306, 0.0, 133.01697, 122.20514, 0.0, 152.75891, 0.0, 126.05996, 112.76912, 0.0, 138.11069, 0.0, 125.67789, 129.37666, 0.0, 143.43307, 0.0, 127.76359, 112.98336, 0.0, 88.16121, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let res_scales :Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.030070057, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.018447906, 0.0, 0.0, 0.0, 0.0, 0.01711597, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.013020085, 0.0, 0.0, 0.0, 0.0, 0.010685049, 0.0, 0.0, 0.0, 0.0, 0.013996841, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.011939427, 0.0, 0.0, 0.0, 0.0, 0.017455006, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.010944449, 0.0, 0.0, 0.0, 0.0, 0.022365179, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let res_zeros : Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 122.60011, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 133.32806, 0.0, 0.0, 0.0, 0.0, 130.66522, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 135.10071, 0.0, 0.0, 0.0, 0.0, 136.239, 0.0, 0.0, 0.0, 0.0, 142.34561, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 128.1635, 0.0, 0.0, 0.0, 0.0, 124.506805, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 120.53104, 0.0, 0.0, 0.0, 0.0, 127.94742, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let mut m = 0.;
    let mut s1 = scales[layer_id];
    let s2 = weight_scales[layer_id];
    let mut s3 = scales[layer_id + 1];
    if zero_points[layer_id + 2] == 0. && scales[layer_id + 2] != 0. {s3 = scales[layer_id + 2];} // skip the relu6
    else if res_scales[layer_id] != 0.0 { //residual connection M  = S1 * S2 / S3
        s3 = res_scales[layer_id];
    }
    m = s1 * s2 / s3;
    let mut zero1 = zero_points[layer_id].round() as u8;
    let zero2  = weight_zero_points[layer_id];
    if zero2 == 0. { panic!("weights not get") }
    let mut zero3 = zero_points[layer_id + 1].round() as u8;
    if zero_points[layer_id + 2] == 0. && scales[layer_id + 2] != 0. {
        zero3 = zero_points[layer_id + 2].round() as u8;
    }
    else if res_scales[layer_id] != 0.0 { //residual connection M  = S1 * S2 / S3
        zero3 = res_zeros[layer_id].round() as u8;
    }
    let quant_weights = original_weights.into_iter().map(|x|{
        x.into_iter().map(|y|{
            QuantizedWeightUnit{
                data: y.data.into_iter().map(|i| (i / s2 + zero2).round().clamp(0.,255.) as u8).collect(),
                bias: (y.bias / (s1 * s2)).round() as i32,
                which_kernel: y.which_kernel,
                count: y.count,
                start_pos_in: y.start_pos_in,
                info: y.info,
                zero_points: (zero1, zero2.round() as u8, zero3),
                m: m,
                s_out:s3 ,
            }
        }).collect::<Vec<QuantizedWeightUnit>>()
    }).collect::<Vec<Vec<QuantizedWeightUnit>>>();
    let quant_mapping = original_mapping.into_iter().map(|x| QuantizedMapping{
        count: x.count,
        map: x.map,
        padding_pos: x.padding_pos,
        end_pos: x.end_pos,
        zero_point: (zero1,zero2.round() as u8,zero3),
        scale: (s1,s2,s3),
    }
    ).collect::<Vec<QuantizedMapping>>();
    (quant_weights, quant_mapping)
}
