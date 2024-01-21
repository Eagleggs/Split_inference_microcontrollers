use crate::lib::{ConvMapping, InfoWrapper, Layer};
use std::ops::{BitAnd, BitOr};

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
) -> Vec<Vec<(Vec<f64>, i32)>> {
    let output_count: i32 = layer
        .get_output_shape()
        .into_iter()
        .fold(1, |acc, x| acc * x as i32);
    let num_per_cpu: i32 = (output_count as f64 / total_cpu_count as f64).ceil() as i32;
    let output_shape = layer.get_output_shape();
    let mut weight_to_send: Vec<Vec<(Vec<f64>, i32)>> = vec![Vec::new(); total_cpu_count as usize];
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
pub fn distributed_convolution(input_distribution:&Vec<f64>,weight_distriution:&Vec<(Vec<f64>,i32)>,info:InfoWrapper)->Vec<f64>{
    todo!();
}