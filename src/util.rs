use std::cmp::max;
use crate::lib::{ConvMapping, InfoWrapper, Layer};
use std::ops::{BitAnd, BitOr};
use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct WeightUnit {
    data: Vec<f64>,
    which_kernel : u16,
    count : i16,
    info : InfoWrapper,
}
pub fn sample_input_from_p_zero_padding(p: Vec<Vec<i16>>, input: &Vec<Vec<Vec<f64>>>) -> Vec<f64> {
    let mut result = Vec::new();
    for i in 0..p.len() {
        let a = &p[i];
        if a[0] < 0
            || a[1] < 0
            || a[2] < 0
            || a[0] >= input.len() as i16
            || a[1] >= input[0].len() as i16
            || a[2] >= input[0][0].len() as i16
        {
            result.push(0.);
        } else {
            result.push(input[a[0] as usize][a[1] as usize][a[2] as usize]);
        }
    }
    result
}
pub fn sample_input_linear(p: Vec<Vec<i16>>, input: &Vec<Vec<f64>>) -> Vec<f64> {
    let mut result = Vec::new();
    for i in 0..p.len() {
        let a = &p[i];
        result.push(input[a[0] as usize][a[1] as usize]);
    }
    result
}
pub fn distribute_weight(
    layer: &Box<dyn Layer>,
    total_cpu_count: i16,
) -> Vec<Vec<WeightUnit>> {
    let output_count: i32 = layer
        .get_output_shape()
        .into_iter()
        .fold(1, |acc, x| acc * x as i32);
    let num_per_cpu: i32 = (output_count as f64 / total_cpu_count as f64).ceil() as i32;
    let output_shape = layer.get_output_shape();
    let mut weight_to_send: Vec<Vec<WeightUnit>> = vec![Vec::new(); total_cpu_count as usize];
    let mut count = 0;
    let mut which_cpu = 0;
    let mut new_kernel_flag = false;
    let mut kernel_data: WeightUnit = WeightUnit{
        data: Vec::new(),
        which_kernel: 0,
        count: 0,
        info: layer.get_info(),
    };
    for j in 0..output_shape[0] {
        new_kernel_flag = true;
        for k in 0..output_shape[1] {
            for m in 0..output_shape[2] {
                if count / num_per_cpu != which_cpu {
                    weight_to_send[which_cpu as usize].push(kernel_data.clone());
                    which_cpu += 1;
                    kernel_data.count = 0;
                }
                let pos = layer.get_input(vec![j, k, m]);
                if new_kernel_flag {
                    if !kernel_data.data.is_empty() {
                        weight_to_send[which_cpu as usize].push(kernel_data.clone());
                    }
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
    return weight_to_send;
}
pub fn get_input_mapping(
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
                let bit_coding: u16 = 1 << which_cpu;
                for p in 0..pos.len() {
                    //-1 will be rounded to a very large value, so no need to check < 0
                    let i: usize = pos[p][0] as usize;
                    let j: usize = (pos[p][1] + 1) as usize; // zero padding
                    let k: usize = (pos[p][2] + 1) as usize;
                    // if i >= input_shape.0 || j >= input_shape.1 || k >= input_shape.2 {
                    //     println!("{},{},{},{},{},{}",i,j,k,input_shape.0,input_shape.1,input_shape.2);
                    // }
                    mapping[i][j][k] = mapping[i][j][k].bitor(bit_coding);
                    if j > input_shape.1 || j == 0 || k > input_shape.2 || k == 0 {
                        mapping[i][j][k] = mapping[i][j][k].bitor(0b1000_0000_0000_0000);
                        // mark this as a padding position;
                    }
                }
                count += 1;
            }
        }
    }
    return mapping;
}
pub fn distribute_input(input:Vec<Vec<Vec<f64>>>,mapping:Vec<Vec<Vec<u16>>>,total_cpu_count:i16)->Vec<Vec<f64>>{
    let mut inputs_distribution = vec![Vec::new();total_cpu_count as usize];
    let mut cpu_to_send_to = Vec::new();
    for i in 0..mapping.len() {
        for j in 0..mapping[0].len() {
            //0 padding
            for k in 0..mapping[0][0].len() {
                let cpu_mapped_to = mapping[i][j][k];
                let padding_flag = cpu_mapped_to >> 15;
                for a in 0..total_cpu_count {
                    let temp = 0b1 << a;
                    if temp.bitand(cpu_mapped_to) == temp {
                        cpu_to_send_to.push(a);
                    }
                }
                if padding_flag == 1 {
                    cpu_to_send_to
                        .iter()
                        .for_each(|&x| inputs_distribution[x as usize].push(0.));
                } else {
                    cpu_to_send_to.iter().for_each(|&x| {
                        inputs_distribution[x as usize].push(input[i][j - 1][k - 1])
                    });
                }
                cpu_to_send_to.clear();
            }
        }
    }
    return inputs_distribution
}
pub fn distributed_computation(input_distribution:Vec<f64>,mut weight_distribution: Vec<WeightUnit>)->Vec<f64>{
    let mut result = Vec::new();

    match &weight_distribution.clone()[0].info{
       InfoWrapper::Convolution(convMapping) =>{
            let mut prev_group = weight_distribution[0].which_kernel / convMapping.o_pg as u16;
            for i in 0..weight_distribution.len(){
                let switch_group = weight_distribution[i].which_kernel / convMapping.o_pg as u16 != prev_group;
                let mut start_point = 0;
                while weight_distribution[i].count > 0{
                    let mut acc = 0.;
                    for c  in 0..convMapping.i_pg{
                        let channel = c * convMapping.i.1 * convMapping.i.2;
                        for j in 0..convMapping.k.0{
                            let col =  j * convMapping.i.2;
                            for k in 0..convMapping.k.1{
                                let row =  k;
                                acc += &input_distribution[(channel + col + row + start_point) as usize] * &weight_distribution[i].data[(c * convMapping.k.0 * convMapping.k.1 + j * convMapping.k.1 + k)  as usize];
                            }
                        }
                    }
                    result.push(acc);
                    start_point += convMapping.s.0;
                    let cur_col = start_point / convMapping.i.2;
                    //edge cases
                    if (start_point + convMapping.k.0) - cur_col * convMapping.i.2 > convMapping.i.2{
                        start_point = (cur_col + convMapping.s.1) * convMapping.i.2;
                    }
                    weight_distribution[i].count -= 1;
                }
            }
        }
        _ => {}
    };

    result
}