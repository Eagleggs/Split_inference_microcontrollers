use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConvMapping {
    map_weights: Vec<Vec<Vec<f64>>>,
    o_i_mapping: String,
}

pub fn json_to_weights() {
    // Read the JSON file
    let mut file = File::open("serialized_list.json").expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");

    // Deserialize the JSON data
    let conv_mapping: Vec<ConvMapping> = serde_json::from_str(&contents).expect("Unable to deserialize JSON");
    let temp = &conv_mapping[0];
    // Now you can use the deserialized data as needed
    for mapping in conv_mapping {
        println!("{:?}", mapping);
    }
}
