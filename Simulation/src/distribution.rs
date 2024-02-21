use std::collections::HashMap;
use std::fmt::format;
use algo::{Layer, operations};
use algo::operations::{distribute_weight, get_input_mapping, mark_end};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
pub fn distribute_mapping_weight(layers:HashMap<i32,Box<dyn Layer>>,number_of_workers: u8,input_shape:(usize,usize,usize),output_dir:String){
    if !fs::metadata(&output_dir).is_ok() {
        // If it doesn't exist, create the folder
        match fs::create_dir_all(&output_dir) {
            Ok(_) => println!("Folder created successfully"),
            Err(e) => eprintln!("Error creating folder: {}", e),
        }
    }
    let mut input_shape = vec![input_shape.0, input_shape.1, input_shape.2];
    for i in 0..layers.len() {
        let layer = layers.get(&(i as i32)).expect("getting layer failed");
        let weight = distribute_weight(layer,number_of_workers);
        let raw_mapping = get_input_mapping(layer,number_of_workers,input_shape);
        let e_pos = mark_end(&raw_mapping, number_of_workers);
        let mappings = operations::analyse_mapping(
            raw_mapping.clone(),
            number_of_workers,
            number_of_workers,
            e_pos,
        );
        input_shape = layer.get_output_shape().into_iter().map(|x| x as usize).collect();
        match layer.identify() {
            "Convolution" =>{
                for i in 0..number_of_workers{
                    let serialized_weights = serde_json::to_string(&weight[i as usize]).unwrap();
                    let file_name = format!("worker_{:?}.json",i);
                    // Write the JSON string to a file
                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(output_dir.to_string() + &file_name)
                        .unwrap();
                    writeln!(file, "{}", serialized_weights).unwrap();
                }
                let serialized_mapping = serde_json::to_string(&mappings).unwrap();
                let file_name = "Coordinator".to_string();
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output_dir.to_string() + &file_name)
                    .unwrap();
                writeln!(file, "{}", serialized_mapping).unwrap();
            }
            "Batchnorm2d" => {
                let serialized_batdata = serde_json::to_string(&weight[0][0].data).unwrap();
                let file_name = format!("Coordinator_batch_norm{:?}.json",i);
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output_dir.to_string() + &file_name)
                    .unwrap();
                writeln!(file, "{}", serialized_batdata).unwrap();
            }
            "Relu6" => {}
            _ => {}
        }
    }
}