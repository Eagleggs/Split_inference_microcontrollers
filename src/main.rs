extern crate core;

use crate::lib::Layer;
use std::fs::File;
use std::time::Instant;
mod calculations;
mod decode;
mod lib;
mod operations;
mod util;

pub fn main() {
    let file = File::open("json_files/test_all.json").expect("Failed to open file");
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
    print!("Finished!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::fs::OpenOptions;
    use std::io::{BufRead, BufReader, Write};
    use std::ops::BitOr;
    use std::thread;
    use crate::lib::ConvMapping;

    #[test]
    fn test_convolution() {
        //weight data
        let file = File::open("json_files/test_convolution.json").expect("Failed to open file");
        let result = decode::decode_json(file);
        let r = result.get(&1).expect("failed");
        let output_shape = r.get_output_shape();
        //input
        let width = 44;
        let height = 44;
        let channels = 3;
        let mut data: Vec<Vec<Vec<f64>>> = Vec::with_capacity(channels);
        for _ in 0..channels {
            let mut channel: Vec<Vec<f64>> = Vec::with_capacity(width);
            for i in 0..height {
                channel.push(vec![i as f64; width]);
            }
            data.push(channel);
        }
        //reference output
        let file = File::open("test_references/conv.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f64> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f64>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }

        for i in 0..output_shape[0] {
            for j in 0..output_shape[1] {
                for m in 0..output_shape[2] {
                    let pos = vec![i, j, m];
                    let inputs_p = r.get_input(pos);
                    let weights: Vec<f64> = r.get_weights_from_input(inputs_p.clone(), i);
                    let inputs = util::sample_input_from_p_zero_padding(inputs_p, &data);
                    let result = calculations::vector_mul_b(inputs, weights, 0.);
                    assert!(
                        (result
                            - reference[(i * output_shape[1] * output_shape[2]
                                + j * output_shape[2]
                                + m) as usize])
                            .abs()
                            < 1e-4
                    )
                }
            }
        }
    }
    #[test]
    fn test_linear() {
        let file = File::open("json_files/test_linear.json").expect("Failed to open file");
        let result = decode::decode_json(file);
        let r = result.get(&141).expect("failed");
        let output_shape = r.get_output_shape();

        //reference output
        let file = File::open("test_references/linear_output.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f64> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f64>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }
        //reference input
        let file = File::open("test_references/linear_input.txt").expect("f");
        let reader = BufReader::new(file);
        let mut input: Vec<Vec<f64>> = Vec::new();
        for line in reader.lines() {
            let temp = line
                .expect("line read failed")
                .split(|x| x == ' ')
                .map(|x| x.parse::<f64>().unwrap())
                .collect::<Vec<f64>>();
            input.push(temp);
        }

        for i in 0..output_shape[0] {
            for j in 0..output_shape[1] {
                let pos = vec![i, j];
                let inputs_p = r.get_input(pos);
                let weights: Vec<f64> = r.get_weights_from_input(inputs_p.clone(), j);
                let bias = r.get_bias(j);
                let inputs = util::sample_input_linear(inputs_p, &input);
                let result = calculations::vector_mul_b(inputs, weights, bias);
                assert!((result - reference[(i * output_shape[1] + j) as usize]).abs() < 1e-4)
            }
        }
        println!("!");
    }
    #[test]
    fn test_conv_norm_relu() {
        //weight data
        let file = File::open("json_files/test_cbr.json").expect("Failed to open file");
        let layers = decode::decode_json(file);
        //input
        let width = 44;
        let height = 44;
        let channels = 3;
        let mut input: Vec<Vec<Vec<f64>>> = Vec::with_capacity(channels);
        for _ in 0..channels {
            let mut channel: Vec<Vec<f64>> = Vec::with_capacity(width);
            for i in 0..height {
                channel.push(vec![i as f64; width]);
            }
            input.push(channel);
        }

        //reference output
        let file = File::open("test_references/cbr_reference_out.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f64> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f64>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }

        for i in 1..=layers.len() {
            let layer = layers.get(&(i as i16)).expect("getting layer failed");
            let output_shape = layer.get_output_shape();
            let mut output = vec![
                vec![vec![0.; output_shape[2] as usize]; output_shape[1] as usize];
                output_shape[0] as usize
            ];
            match layer.identify() {
                "Convolution" => {
                    let mut flag = true;
                    for j in 0..output_shape[0] as usize {
                        flag = true;
                        let mut weights: Vec<f64> = Vec::new();
                        for k in 0..output_shape[1] as usize {
                            for m in 0..output_shape[2] as usize {
                                let pos = vec![j as i16, k as i16, m as i16];
                                let inputs_p = layer.get_input(pos);
                                //each output channel only need to sample weight once
                                if flag {
                                    weights =
                                        layer.get_weights_from_input(inputs_p.clone(), j as i16);
                                    flag = false;
                                }
                                let inputs =
                                    util::sample_input_from_p_zero_padding(inputs_p, &input);
                                let result =
                                    calculations::vector_mul_b(inputs, weights.clone(), 0.);
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
                _ => {}
            }
        }
        for i in 0..input.len() {
            for j in 0..input[0].len() {
                for k in 0..input[0][0].len() {
                    assert!(
                        (input[i][j][k]
                            - reference[i * input[0].len() * input[0][0].len()
                                + j * input[0][0].len()
                                + k])
                            .abs()
                            < 1e-4
                    )
                }
            }
        }
    }
    #[test]
    fn test_residual_connection() {
        //residual connections for mobilenet v2
        let residual_connections = vec![
            vec![16, 24],
            vec![32, 40],
            vec![40, 48],
            vec![56, 64],
            vec![64, 72],
            vec![72, 80],
            vec![88, 96],
            vec![96, 104],
            vec![112, 120],
            vec![120, 128],
        ];

        //weight data
        let mut start_time = Instant::now();
        let file = File::open("json_files/test_residual.json").expect("Failed to open file");
        let layers = decode::decode_json(file);
        let mut end_time = Instant::now();
        let mut elapsed_time = end_time - start_time;
        // Print the result and elapsed time
        println!("decoding file time,elapsed time: {:?}", elapsed_time);
        //input
        let width = 44;
        let height = 44;
        let channels = 3;
        let mut input: Vec<Vec<Vec<f64>>> = Vec::with_capacity(channels);
        for _ in 0..channels {
            let mut channel: Vec<Vec<f64>> = Vec::with_capacity(width);
            for i in 0..height {
                channel.push(vec![i as f64; width]);
            }
            input.push(channel);
        }

        //reference output
        let file = File::open("test_references/residual_reference_out.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f64> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f64>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }

        let mut intermediate_output: Vec<Vec<Vec<Vec<f64>>>> = Vec::new();
        start_time = Instant::now();
        for i in 1..=layers.len() {
            let layer = layers.get(&(i as i16)).expect("getting layer failed");
            let output_shape = layer.get_output_shape();
            let mut output = vec![
                vec![vec![0.; output_shape[2] as usize]; output_shape[1] as usize];
                output_shape[0] as usize
            ];
            match layer.identify() {
                "Convolution" => {
                    let mut flag = true;
                    for j in 0..output_shape[0] as usize {
                        flag = true;
                        let mut weights: Vec<f64> = Vec::new();
                        for k in 0..output_shape[1] as usize {
                            for m in 0..output_shape[2] as usize {
                                let pos = vec![j as i16, k as i16, m as i16];
                                let inputs_p = layer.get_input(pos);
                                //each output channel only need to sample weight once
                                if flag {
                                    weights =
                                        layer.get_weights_from_input(inputs_p.clone(), j as i16);
                                    flag = false;
                                }
                                let inputs =
                                    util::sample_input_from_p_zero_padding(inputs_p, &input);
                                let result =
                                    calculations::vector_mul_b(inputs, weights.clone(), 0.);
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
                _ => {}
            }
            for r in 0..residual_connections.len() {
                if residual_connections[r][0] == i {
                    intermediate_output.push(input.clone());
                }
                if residual_connections[r][1] == i {
                    for j in 0..output_shape[0] as usize {
                        for k in 0..output_shape[1] as usize {
                            for m in 0..output_shape[2] as usize {
                                input[j][k][m] += intermediate_output[r][j][k][m];
                            }
                        }
                    }
                }
            }
        }
        end_time = Instant::now();
        elapsed_time = end_time - start_time;
        println!("Forward pass time,elapsed time: {:?}", elapsed_time);
        for i in 0..input.len() {
            for j in 0..input[0].len() {
                for k in 0..input[0][0].len() {
                    if (input[i][j][k]
                        - reference
                            [i * input[0].len() * input[0][0].len() + j * input[0][0].len() + k])
                        .abs()
                        >= 1e-4
                    {
                        println!(
                            "left:{:?},right:{:?}",
                            input[i][j][k],
                            reference[i * input[0].len() * input[0][0].len()
                                + j * input[0][0].len()
                                + k]
                        );
                    }
                    assert!(
                        (input[i][j][k]
                            - reference[i * input[0].len() * input[0][0].len()
                                + j * input[0][0].len()
                                + k])
                            .abs()
                            < 1e-4
                    )
                }
            }
        }
    }
    #[test]
    fn test_weight_distribution() {
        let total_cpu_count = 5;
        let residual_connections = vec![
            vec![16, 24],
            vec![32, 40],
            vec![40, 48],
            vec![56, 64],
            vec![64, 72],
            vec![72, 80],
            vec![88, 96],
            vec![96, 104],
            vec![112, 120],
            vec![120, 128],
        ];
        let file = File::open("json_files/test_residual.json").expect("Failed to open file");
        let layers = decode::decode_json(file);
        fn distribute_weight(
            layer: &Box<dyn Layer>,
            total_cpu_count: i32,
        ) -> Vec<Vec<(Vec<f64>, i32)>> {
            let output_count: i32 = layer
                .get_output_shape()
                .into_iter()
                .fold(1, |acc, x| acc * x as i32);
            let num_per_cpu: i32 = (output_count as f64 / total_cpu_count as f64).ceil() as i32;
            let output_shape = layer.get_output_shape();
            let mut weight_to_send: Vec<Vec<(Vec<f64>, i32)>> =
                vec![Vec::new(); total_cpu_count as usize];
            let mut count = 0;
            let mut which_cpu = 0;
            let mut new_kernel_flag = false;
            let mut kernel_data: (Vec<f64>, i32) = (Vec::new(), 0);
            for j in 0..output_shape[0] {
                new_kernel_flag = true;
                for k in 0..output_shape[1] {
                    for m in 0..output_shape[2] {
                        if count / num_per_cpu != which_cpu {
                            weight_to_send[which_cpu as usize].push(kernel_data.clone());
                            which_cpu += 1;
                            kernel_data.1 = 0;
                        }
                        let pos = layer.get_input(vec![j, k, m]);
                        if new_kernel_flag {
                            if !kernel_data.0.is_empty() {
                                weight_to_send[which_cpu as usize].push(kernel_data.clone());
                            }
                            kernel_data.0 = layer.get_weights_from_input(pos, j);
                            new_kernel_flag = false;
                            kernel_data.1 = 0;
                        }
                        kernel_data.1 += 1;
                        count += 1;
                    }
                }
            }
            return weight_to_send;
        }
        fn get_input_mapping(
            layer: &Box<dyn Layer>,
            total_cpu_count: i16,
            input_shape: (usize, usize, usize),
        ) -> Vec<Vec<Vec<u16>>> {
            let output_count: i32 = layer
                .get_output_shape()
                .into_iter()
                .fold(1, |acc, x| acc * x as i32);
            let num_per_cpu: i32 = (output_count as f64 / total_cpu_count as f64).ceil() as i32;
            let mut start_end_index: Vec<(Vec<i16>, Vec<i16>)> = Vec::new();
            let mut mapping: Vec<Vec<Vec<u16>>> =
                vec![vec![vec![0; input_shape.2 + 2]; input_shape.1 + 2]; input_shape.0]; //zero padding,kernel_size maximum = 3*3;
            let mut count: i32 = 0;
            let output_shape = layer.get_output_shape();
            let mut new_kernel_flag = false;
            let mut which_cpu = 0;
            for j in 0..output_shape[0] {
                for k in 0..output_shape[1] {
                    for m in 0..output_shape[2] {
                        if count / num_per_cpu != which_cpu {
                            which_cpu += 1;
                        }
                        let pos = layer.get_input(vec![j, k, m]);
                        //maximum 16 cpus,because of u16 type
                        let bit_coding: u16 = 1<< which_cpu;
                        for p in 0..pos.len() {
                            //-1 will be rounded to a very large value, so no need to check < 0
                            let i: usize = pos[p][0] as usize;
                            let j: usize = (pos[p][1] + 1) as usize;// zero padding
                            let k: usize = (pos[p][2] + 1) as usize;
                            // if i >= input_shape.0 || j >= input_shape.1 || k >= input_shape.2 {
                            //     println!("{},{},{},{},{},{}",i,j,k,input_shape.0,input_shape.1,input_shape.2);
                            // }
                            mapping[i][j][k] = mapping[i][j][k].bitor(bit_coding);
                            if j > input_shape.1 || j ==0 || k > input_shape.2 || k == 0{
                                mapping[i][j][k] = mapping[i][j][k].bitor(0b1000_0000_0000_0000); // mark this as a padding position;
                            }
                        }
                        count += 1;
                    }
                }
            }
            return mapping;
        }
        let mut input_shape = (3, 44, 44);
        for i in 1..=layers.len() {
            let layer = layers.get(&(i as i16)).unwrap();
            match layer.identify() {
                "Convolution" => {
                    let weight = distribute_weight(layer, 7);
                    let mapping = get_input_mapping(layer, 7, input_shape);
                    let output_shape = layer.get_output_shape();
                    let serialized = serde_json::to_string(&mapping).unwrap();
                    // Write the JSON string to a file
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("output.json")
                        .unwrap();
                    writeln!(file, "{}", serialized).unwrap();
                    input_shape = (
                        output_shape[0] as usize,
                        output_shape[1] as usize,
                        output_shape[2] as usize,
                    );
                }
                _ => {}
            }
        }
    }
}
