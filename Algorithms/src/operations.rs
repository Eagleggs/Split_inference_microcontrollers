use crate::lib::{InfoWrapper, Layer};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::ops::{BitAnd, BitOr};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeightUnit {
    data: Vec<f64>,
    which_kernel: u16,
    count: i32,
    start_pos_in: Vec<i32>,
    info: InfoWrapper,
}
pub fn distribute_weight(layer: &Box<dyn Layer>, total_cpu_count: i32) -> Vec<Vec<WeightUnit>> {
    let output_shape = layer.get_output_shape();
    let mut weight_to_send: Vec<Vec<WeightUnit>> = vec![Vec::new(); total_cpu_count as usize];
    let mut count: i32 = 0;
    let mut which_cpu = 0;
    let mut new_kernel_flag = false;
    let mut kernel_data: WeightUnit = WeightUnit {
        data: Vec::new(),
        which_kernel: 0,
        count: 0,
        start_pos_in: vec![],
        info: layer.get_info(),
    };
    match layer.get_info() {
         InfoWrapper::Convolution(conv) =>{
             let output_count: i32 = layer
             .get_output_shape()
             .into_iter()
             .fold(1, |acc, x| acc * x as i32);
             let num_per_cpu: i32 = (output_count as f64 / total_cpu_count as f64).ceil() as i32;
             for j in 0..output_shape[0] {
                 new_kernel_flag = true;
                 for k in 0..output_shape[1] {
                     for m in 0..output_shape[2] {
                         let pos = layer.get_input(vec![j, k, m]);
                         if count / num_per_cpu as i32 != which_cpu {
                             weight_to_send[which_cpu as usize].push(kernel_data.clone());
                             rearrange_weight(&mut weight_to_send[which_cpu as usize]);
                             kernel_data.start_pos_in = pos[0].clone();
                             which_cpu += 1;
                             kernel_data.count = 0;
                         }
                         if new_kernel_flag {
                             if !kernel_data.data.is_empty() {
                                 weight_to_send[which_cpu as usize].push(kernel_data.clone());
                             }
                             kernel_data.start_pos_in = pos[0].clone();
                             kernel_data.data = layer.get_weights_from_input(pos, j);
                             kernel_data.which_kernel = j as u16;
                             new_kernel_flag = false;
                             kernel_data.count = 0;
                         }
                         kernel_data.count += 1;
                         count += 1;
                     }
                 }
             }

             weight_to_send[which_cpu as usize].push(kernel_data.clone());
             rearrange_weight(&mut weight_to_send[which_cpu as usize]);            ;
        }
        InfoWrapper::Linear(info) =>{
            let weight = layer.get_weights();
            let weight_shape = vec![info.c_in,info.c_out]; //1280,1000
            let col_per_cpu = (weight_shape[1] as f64 / total_cpu_count as f64).ceil() as i32;
            for j in 0..weight_shape[1]{
                for k in 0..weight_shape[0]{
                    kernel_data.data.push(weight[(k * weight_shape[0] + j) as usize]);
                }
                kernel_data.which_kernel = j as u16;
                which_cpu = j / col_per_cpu;
                weight_to_send[which_cpu as usize].push(kernel_data.clone());
                kernel_data.data.clear();
            };
        }
        InfoWrapper::ReLU6(info) => {
            weight_to_send.resize(0,vec![]); // no weight data
        }
        InfoWrapper::BatchNorm2d(info) => { //store in the coordinator, so size = 1
            weight_to_send.resize(1, vec![]);
            kernel_data.data = layer.get_weights();
            weight_to_send[0] = vec![kernel_data];
        }
    }
    weight_to_send
}
pub fn get_input_mapping(
    layer: &Box<dyn Layer>,
    total_cpu_count: i32,
    input_shape: Vec<usize>,
) -> Vec<Vec<Vec<u16>>> {
    let output_count: i32 = layer
        .get_output_shape()
        .into_iter()
        .fold(1, |acc, x| acc * x as i32);
    let num_per_cpu: i32 = (output_count as f64 / total_cpu_count as f64).ceil() as i32;
    let mut mapping = vec![];
    match layer.get_info() {
        InfoWrapper::Convolution(conv) =>{
            let mut kernel_size: (u16, u16) = (0, 0);
            kernel_size = (conv.k.0 as u16, conv.k.1 as u16);
            let padding_numbers = (kernel_size.0 / 2 * 2, kernel_size.1 / 2 * 2);
            mapping =
                vec![
                    vec![
                        vec![0; input_shape[2] + padding_numbers.1 as usize];
                        input_shape[1] + padding_numbers.0 as usize
                    ];
                    input_shape[0]
                ]; //zero padding,kernel_size maximum = 3*3;
            let mut count: i32 = 0;
            let output_shape = layer.get_output_shape();
            let mut which_cpu = 0;
            for j in 0..output_shape[0] {
                for k in 0..output_shape[1] {
                    for m in 0..output_shape[2] {
                        if count / num_per_cpu as i32 != which_cpu {
                            which_cpu += 1;
                        }
                        let pos = layer.get_input(vec![j, k, m]);
                        //maximum 16 cpus,because of u16 type
                        let bit_coding: u16 = 1 << which_cpu;
                        for p in 0..pos.len() {
                            //-1 will be rounded to a very large value, so no need to check < 0
                            let a: usize = pos[p][0] as usize;
                            let b: usize = (pos[p][1] + (padding_numbers.0 / 2) as i32) as usize; // zero padding
                            let c: usize = (pos[p][2] + (padding_numbers.1 / 2) as i32) as usize;
                            mapping[a][b][c] = mapping[a][b][c].bitor(bit_coding);
                            if (b > input_shape[1] || b == 0) && padding_numbers.0 != 0
                                || (c > input_shape[2] || c == 0) && padding_numbers.1 != 0
                            {
                                mapping[a][b][c] = mapping[a][b][c].bitor(0b1000_0000_0000_0000);
                                // mark this as a padding position;
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
        InfoWrapper::ReLU6(info) =>{}
        InfoWrapper::BatchNorm2d(info) => {}
        InfoWrapper::Linear(info) =>{} // full pass
    }
    //empty mapping means full pass
    mapping
}
pub fn distribute_input(input: Vec<Vec<Vec<f64>>>,
                            mapping: Vec<Vec<Vec<u16>>>,
                            total_cpu_count: i32,
) -> Vec<Vec<f64>>{
    if mapping.is_empty() { return vec![]} //full pass
    let mut inputs_distribution = vec![Vec::new(); total_cpu_count as usize];
    let mut i_x = 0;
    let mut i_y = 0;
    for i in 0..mapping.len(){
        for j in 0..mapping[0].len(){
            for m in 0..mapping[0][0].len(){
                let map = mapping[i][j][m];
                if map == 0 { continue }
                let padding_flag = map >> 15 == 0b1;
                let mut cpu_mapped_to = Vec::new();
                for k in 0..15{
                    if (map >> k).bitand(0b1) == 0b1{
                        cpu_mapped_to.push(k);
                    }
                }
                for a in cpu_mapped_to{
                    if padding_flag{ inputs_distribution[a as usize].push(0.)}
                    else {
                        inputs_distribution[a as usize].push(input[i][i_y][i_x]);
                    }
                }
                if !padding_flag{
                    i_x += 1;
                    if i_x == input[0][0].len() {
                        i_x = 0;
                        i_y += 1;
                        if i_y == input[0].len() {
                            i_x = 0;
                            i_y = 0;
                        }
                    }
                }
            }
        }
    }
    inputs_distribution
}
pub fn distributed_computation(
    input_distribution: Vec<f64>,
    mut weight_distribution: Vec<WeightUnit>,
) -> Vec<f64> {
    let mut result = vec![Vec::new(); 10000];
    match &weight_distribution.clone()[0].info {
        InfoWrapper::Convolution(convMapping) => {
            let mut start_point = 0;
            let mut max_visited = weight_distribution[0].start_pos_in.clone();
            let mut prev_kernel_nr = 0;
            let mut first_row = false;
            let mut out_side_rows = 0;
            let mut in_side_rows = 0;
            for i in 0..weight_distribution.len() {
                let mut padded_row = weight_distribution[i].start_pos_in[1] + convMapping.k.0 / 2;
                let mut padded_col = weight_distribution[i].start_pos_in[2] + convMapping.k.1 / 2;
                let mut adjustment = 0;
                if weight_distribution[i].count == 0 {
                    continue;
                }
                //handel heads
                if i == 0 && first_row == false {
                    first_row = true;
                    if convMapping.i.2 - padded_row <= convMapping.k.1 {
                        // assuming at least 2 rows can be stored
                        out_side_rows = convMapping.k.1;
                    } else {
                        out_side_rows = convMapping.s.1;
                    }
                    adjustment = padded_col;
                    in_side_rows = convMapping.k.1 - out_side_rows;
                }
                //switch group
                if weight_distribution[i].start_pos_in > max_visited {
                    let rows_to_move_down = convMapping.k.1 - convMapping.s.1; // the last calculation will always move down a stride
                    start_point = start_point
                        + rows_to_move_down * convMapping.i.2
                        + (convMapping.i_pg - 1) * convMapping.i.1 * convMapping.i.2;
                } else {
                    // change within same group
                    let prev_end_pos = &weight_distribution[i.saturating_sub(1)].start_pos_in;
                    let diff = weight_distribution[i]
                        .start_pos_in
                        .iter()
                        .zip(prev_end_pos.iter())
                        .map(|(x, y)| y - x)
                        .collect::<Vec<i32>>();
                    start_point = start_point - diff[1] * convMapping.i.2 - diff[2];
                }

                while weight_distribution[i].count > 0 {
                    padded_row = weight_distribution[i].start_pos_in[1] + convMapping.k.0 / 2;
                    padded_col = weight_distribution[i].start_pos_in[2] + convMapping.k.1 / 2;
                    let mut acc = 0.;
                    for c in 0..convMapping.i_pg {
                        let channel = c * convMapping.i.1 * convMapping.i.2;
                        for j in 0..convMapping.k.0 {
                            let col = j * convMapping.i.2;
                            for k in 0..convMapping.k.1 {
                                let row = k;
                                let mut index = (channel + col + row + start_point) as usize;

                                let remaining = input_distribution.len() as i32 - start_point;

                                let mut inside_rows = convMapping.k.1 - out_side_rows;
                                let to_complete = convMapping.k.1 * convMapping.i.2 - padded_col;
                                //handel tails
                                if remaining < to_complete && !first_row {
                                    if padded_row >= convMapping.s.1 {
                                        out_side_rows = convMapping.s.1;
                                    } else {
                                        out_side_rows = convMapping.k.1;
                                    }
                                    inside_rows = convMapping.k.0 - out_side_rows; //can not fill the gap, handel this in the bracket
                                    let empty_pos = (to_complete - remaining) / out_side_rows;
                                    if j > inside_rows {
                                        index -= (j - inside_rows) as usize * empty_pos as usize;
                                    }
                                }
                                //handel heads
                                else if first_row && remaining >= to_complete {
                                    if j < out_side_rows {
                                        index -= j as usize * adjustment as usize;
                                    } else {
                                        index -= (out_side_rows - 1) as usize * adjustment as usize;
                                    }
                                } else if first_row && remaining < to_complete {
                                    //all input distributions are within the same row
                                    panic!("not implemented yet");
                                }
                                acc += &input_distribution[index]
                                    * &weight_distribution[i].data[(c
                                        * convMapping.k.0
                                        * convMapping.k.1
                                        + j * convMapping.k.1
                                        + k)
                                        as usize];
                            }
                        }
                    }

                    result[weight_distribution[i].which_kernel as usize].push(acc);
                    prev_kernel_nr = weight_distribution[i].which_kernel;
                    weight_distribution[i].start_pos_in[2] += convMapping.s.0;
                    start_point += convMapping.s.0;
                    //change a column
                    if weight_distribution[i].start_pos_in[2]
                        + convMapping.k.0 / 2
                        + convMapping.k.0
                        > convMapping.i.2
                    {
                        weight_distribution[i].start_pos_in[2] = 0 - convMapping.k.0 / 2; //zero padding
                        weight_distribution[i].start_pos_in[1] += convMapping.s.1;

                        start_point = start_point - convMapping.s.0
                            + convMapping.k.0
                            + ((convMapping.s.1 - 1) * convMapping.i.1); // move to next row, first move left to the last position calculated, then add kernel size, then move down
                        if first_row {
                            start_point -= (out_side_rows - 1) * adjustment;
                            first_row = false;
                        }
                    }
                    max_visited = max(max_visited, weight_distribution[i].start_pos_in.clone());
                    weight_distribution[i].count -= 1;
                }
            }
        }
        InfoWrapper::ReLU6(info)=> {
            result[0] = input_distribution.into_iter().map(|x| x.clamp(0.,6.0)).collect::<Vec<f64>>();
        }
        InfoWrapper::Linear(info)=>{
            for w in weight_distribution{
                let p = w.which_kernel;
                let r = w.data.into_iter().zip(input_distribution.iter()).fold(0.0,|acc,(x,y)| acc + x * y);
                result[p as usize].push(r);
            }
        }
        InfoWrapper::BatchNorm2d(info)=>{}
    };
    result.concat()
}
pub fn rearrange_weight(weight: &mut Vec<WeightUnit>) {
    weight.sort_by(|x, y| x.start_pos_in.cmp(&y.start_pos_in));
}