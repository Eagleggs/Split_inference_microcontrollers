use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConvMapping {
    weights: Vec<Vec<Vec<f64>>>,
    mapping: OIMapping,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OIMapping {
    i_ch_s: String,
    s: Vec<usize>,
    k: Vec<usize>,
}
pub fn decode_json()-> HashMap<String,ConvMapping> {
    // Read the JSON file into a string
    let mut file = File::open("serialized_list.json").expect("Failed to open file");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)
        .expect("Failed to read file");

    // Deserialize the JSON string into a ConvMapping struct
    let conv_mapping: HashMap<String,ConvMapping> = serde_json::from_str(&json_string).expect("Failed to deserialize JSON");

    // Now you can access the data in conv_mapping
    println!("{:?}", conv_mapping);
    return conv_mapping;
}
