use algo::{decode, Layer};
use std::fs::File;

pub fn main() {
    let file = File::open("Algorithms/json_files/test_17_63.json").expect("Failed to open file");
    let result = decode::decode_json(file);
    // Iterate over the entries and print each key-value pair
    let mut sorted = result.into_iter().collect::<Vec<(i32, Box<dyn Layer>)>>();
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
    use algo::{calculations, operations, util};
    use std::cmp::max;
    use std::env;
    use std::fs::OpenOptions;
    use std::io::{BufRead, BufReader};
    use std::time::Instant;

    #[test]
    fn test_convolution() {
        //weight data
        let file = File::open(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Algorithms\json_files\test_convolution.json").expect("Failed to open file");
        let result = decode::decode_json(file);
        let r = result.get(&1).expect("failed");
        let output_shape = r.get_output_shape();
        //input
        let width = 44;
        let height = 44;
        let channels = 3;
        let mut data: Vec<Vec<Vec<f32>>> = vec![vec![vec![0.; 44]; 44]; 3];

        for c in 0..channels {
            for i in 0..height {
                for j in 0..width {
                    data[c][i][j] = (c * width * height + i * height + j) as f32;
                }
            }
        }
        //reference output
        let file = File::open(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Algorithms\test_references\conv_new.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f32> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f32>() {
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
                    let weights: Vec<f32> = r.get_weights_from_input(inputs_p.clone(), i);
                    let inputs = util::sample_input_from_p_zero_padding(inputs_p, &data);
                    let result = calculations::vector_mul_b(inputs, weights, 0.);
                    assert!(
                        (result
                            - reference[(i * output_shape[1] * output_shape[2]
                                + j * output_shape[2]
                                + m) as usize])
                            .abs()
                            < 1e-2
                    )
                }
            }
        }
    }
    #[test]
    fn test_linear() {
        let file = File::open(".//json_files/test_linear.json").expect("Failed to open file");
        let result = decode::decode_json(file);
        let r = result.get(&141).expect("failed");
        let output_shape = r.get_output_shape();

        //reference output
        let file = File::open(".//test_references/linear_output.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f32> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f32>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }
        //reference input
        let file = File::open(".//test_references/linear_input.txt").expect("f");
        let reader = BufReader::new(file);
        let mut input: Vec<Vec<f32>> = Vec::new();
        for line in reader.lines() {
            let temp = line
                .expect("line read failed")
                .split(|x| x == ' ')
                .map(|x| x.parse::<f32>().unwrap())
                .collect::<Vec<f32>>();
            input.push(temp);
        }

        for i in 0..output_shape[0] {
            for j in 0..output_shape[1] {
                let pos = vec![i, j];
                let inputs_p = r.get_input(pos);
                let weights: Vec<f32> = r.get_weights_from_input(inputs_p.clone(), j);
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
        let file = File::open(".//json_files/test_cbr.json").expect("Failed to open file");
        let layers = decode::decode_json(file);
        //input
        let width = 44;
        let height = 44;
        let channels = 3;
        let mut input: Vec<Vec<Vec<f32>>> = Vec::with_capacity(channels);
        for _ in 0..channels {
            let mut channel: Vec<Vec<f32>> = Vec::with_capacity(width);
            for i in 0..height {
                channel.push(vec![i as f32; width]);
            }
            input.push(channel);
        }

        //reference output
        let file = File::open(".//test_references/cbr_reference_out.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f32> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f32>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }

        for i in 1..=layers.len() {
            let layer = layers.get(&(i as i32)).expect("getting layer failed");
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
                        let mut weights: Vec<f32> = Vec::new();
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
        let file = File::open(".//json_files/test_residual.json").expect("Failed to open file");
        let layers = decode::decode_json(file);
        let mut end_time = Instant::now();
        let mut elapsed_time = end_time - start_time;
        // Print the result and elapsed time
        println!("decoding file time,elapsed time: {:?}", elapsed_time);
        //input

        let width = 44;
        let height = 44;
        let channels = 3;
        let mut input: Vec<Vec<Vec<f32>>> = vec![vec![vec![0.; 44]; 44]; 3];

        for c in 0..channels {
            for i in 0..height {
                for j in 0..width {
                    input[c][i][j] = (c * width * height + i * height + j) as f32;
                }
            }
        }

        //reference output
        let file = File::open(".//test_references/residual_reference_out_new.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f32> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f32>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }

        let mut intermediate_output: Vec<Vec<Vec<Vec<f32>>>> = Vec::new();
        start_time = Instant::now();
        for i in 1..=layers.len() {
            let layer = layers.get(&(i as i32)).expect("getting layer failed");
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
                        let mut weights: Vec<f32> = Vec::new();
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
                        >= 1e-2
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
                            < 1e-2
                    )
                }
            }
        }
    }
    #[test]
    fn test_weight_distribution_single_convolution() {
        let file = File::open(".//json_files/test_convolution.json").expect("Failed to open file");
        let result = decode::decode_json(file);
        let layer = result.get(&1).expect("failed");
        let output_shape = layer.get_output_shape();
        //input
        let width = 44;
        let height = 44;
        let channels = 3;

        let mut input: Vec<Vec<Vec<f32>>> = vec![vec![vec![0.; 44]; 44]; 3];

        for c in 0..channels {
            for i in 0..height {
                for j in 0..width {
                    input[c][i][j] = (c * width * height + i * height + j) as f32;
                }
            }
        }

        let temp = layer.get_info();
        let mut input_shape: Vec<usize> = vec![3, 44, 44];
        let total_cpu_count = 7; //1-15 because of u16 coding for mapping
        let mut weight = operations::distribute_weight(layer, total_cpu_count);
        let mapping = operations::get_input_mapping(layer, total_cpu_count, input_shape);
        let mut inputs_distribution = operations::distribute_input(input, mapping, total_cpu_count);
        let output_shape = layer.get_output_shape();
        let mut output = vec![
            vec![vec![0.; output_shape[2] as usize]; output_shape[1] as usize];
            output_shape[0] as usize
        ];
        let mut output_buffer = Vec::new();
        for i in 0..total_cpu_count as usize {
            let info = layer.get_info();
            let mut result = operations::distributed_computation(
                inputs_distribution[i].clone(),
                weight[i].clone(),
            );
            output_buffer.append(&mut result);
        }
        for i in 0..output_shape[0] as usize {
            for j in 0..output_shape[1] as usize {
                for k in 0..output_shape[2] as usize {
                    output[i][j][k] =
                        output_buffer[i * output_shape[1] as usize * output_shape[2] as usize
                            + j * output_shape[2] as usize
                            + k];
                }
            }
        }

        let file = File::open(".//test_references/conv_new.txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f32> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f32>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }
        for i in 0..output_shape[0] {
            for j in 0..output_shape[1] {
                for m in 0..output_shape[2] {
                    if (output[i as usize][j as usize][m as usize]
                        - reference[(i * output_shape[1] * output_shape[2]
                            + j * output_shape[2]
                            + m) as usize])
                        .abs()
                        >= 1e-2
                    {
                        println!(
                            "{:?},{:?},{:?}",
                            output[i as usize][j as usize][m as usize],
                            reference[(i * output_shape[1] * output_shape[2]
                                + j * output_shape[2]
                                + m) as usize],
                            (i * output_shape[1] * output_shape[2] + j * output_shape[2] + m)
                        )
                    }
                    assert!(
                        (output[i as usize][j as usize][m as usize]
                            - reference[(i * output_shape[1] * output_shape[2]
                                + j * output_shape[2]
                                + m) as usize])
                            .abs()
                            < 1e-2
                    )
                }
            }
        }
        println!("!");
        // let serialized = serde_json::to_string(&mapping).unwrap();
        // // Write the JSON string to a file
        // let mut file = OpenOptions::new()
        //     .create(true)
        //     .append(true)
        //     .open("output.json")
        //     .unwrap();
        // writeln!(file, "{}", serialized).unwrap();
        // input_shape = (
        //     output_shape[0] as usize,
        //     output_shape[1] as usize,
        //     output_shape[2] as usize,
        // );
    }
    #[test]
    fn test_distributed_139() {
        use std::io::Write;
        use std::mem;
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
        let file = File::open("/home/lu/CLionProjects/Split_learning_microcontrollers/Algorithms/json_files/test_17_63.json").expect("Failed to open file");
        let layers = decode::decode_json(file);

        let width = 224;
        let height = 224;
        let channels = 3;
        let mut input: Vec<Vec<Vec<f32>>> = vec![vec![vec![0.; width]; height]; 3];
        let mut input_shape = vec![3, height, width];
        for c in 0..channels {
            for i in 0..height {
                for j in 0..width {
                    input[c][i][j] = (c * width * height + i * height + j) as f32;
                }
            }
        }
        //reference output
        let file = File::open("/home/lu/CLionProjects/Split_learning_microcontrollers/Algorithms/test_references/conv_17_63_txt").expect("f");
        let reader = BufReader::new(file);
        let mut reference: Vec<f32> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("line read failed");
            if let Ok(value) = line.trim().parse::<f32>() {
                reference.push(value);
            } else {
                eprintln!("Error parsing line: {}", line);
            }
        }
        let mut intermediate_output: Vec<Vec<Vec<Vec<f32>>>> = Vec::new();
        let mut maximum_input_size = 0;
        let mut maximum_weight_size = 0;
        let mut total_weight_size = 0;
        for i in 1..=layers.len() {
            if i == 1 {
                println!("!");
            }
            let layer = layers.get(&(i as i32)).expect("getting layer failed");
            let output_shape = layer.get_output_shape();
            let mut output = vec![
                vec![vec![0.; output_shape[2] as usize]; output_shape[1] as usize];
                output_shape[0] as usize
            ];

            match layer.identify() {
                "Convolution" => {
                    let total_cpu_count = 127; //1-32
                    let mut weight = operations::distribute_weight(layer, total_cpu_count);
                    let mapping =
                        operations::get_input_mapping(layer, total_cpu_count, input_shape);

                    // let test = operations::analyse_mapping(mapping.clone(),15,15);
                    // let serialized = serde_json::to_string(&mapping).unwrap();
                    // // Write the JSON string to a file
                    // let mut file = OpenOptions::new()
                    //     .create(true)
                    //     .append(true)
                    //     .open("./output.json")
                    //     .unwrap();
                    // writeln!(file, "{}", serialized).unwrap();
                    let mut inputs_distribution =
                        operations::distribute_input(input, mapping, total_cpu_count);
                    let output_shape = layer.get_output_shape();
                    let mut output =
                        vec![
                            vec![vec![0.; output_shape[2] as usize]; output_shape[1] as usize];
                            output_shape[0] as usize
                        ];
                    let mut output_buffer = Vec::new();
                    for i in 0..total_cpu_count as usize {
                        let info = layer.get_info();
                        maximum_input_size = max(
                            maximum_input_size,
                            std::mem::size_of_val(&inputs_distribution[i][0])
                                * inputs_distribution[i].len(),
                        );
                        let mut size = 0;
                        weight[i].iter().for_each(|x| size += x.data.len() * 4 + 66);
                        maximum_weight_size = max(maximum_weight_size, size);
                        total_weight_size += size;
                        let mut result = operations::distributed_computation(
                            inputs_distribution[i].clone(),
                            weight[i].clone(),
                        );
                        output_buffer.append(&mut result);
                    }
                    for i in 0..output_shape[0] as usize {
                        for j in 0..output_shape[1] as usize {
                            for k in 0..output_shape[2] as usize {
                                output[i][j][k] = output_buffer[i
                                    * output_shape[1] as usize
                                    * output_shape[2] as usize
                                    + j * output_shape[2] as usize
                                    + k];
                                // output[i][j][k] = 0.;
                            }
                        }
                    }
                    input = output;
                    input_shape = vec![
                        output_shape[0] as usize,
                        output_shape[1] as usize,
                        output_shape[2] as usize,
                    ]
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
        println!("maximum input size: {:?} Kbytes", maximum_input_size as f32 / 1024.);
        println!("maximum weight size: {:?} Kbytes", maximum_weight_size as f32 / 1024.);
        println!("total weight size: {:?} Kbytes", total_weight_size as f32 / 1024.);

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
                            "left:{:?},right:{:?},{:?}",
                            input[i][j][k],
                            reference[i * input[0].len() * input[0][0].len()
                                + j * input[0][0].len()
                                + k],vec![i,j,k]
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
}
