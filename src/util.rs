use crate::lib::{ConvMapping, InfoWrapper, Layer};
use serde::{Deserialize, Serialize};
use std::cmp::max;
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

