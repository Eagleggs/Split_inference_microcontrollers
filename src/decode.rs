use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    i: Vec<Vec<i16>>,
    w: Vec<f64>,
}
pub fn decode_json(mut file : File)-> HashMap<String,Mapping> {
    // Read the JSON file into a string
    // let mut file = File::open("test.json").expect("Failed to open file");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)
        .expect("Failed to read file");

    // Deserialize the JSON string into a ConvMapping struct
    let mapping: HashMap<String,Mapping> = serde_json::from_str(&json_string).expect("Failed to deserialize JSON");

    // Now you can access the data in conv_mapping
    return mapping;
}
