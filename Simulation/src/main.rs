extern crate core;

mod distribution;
mod nodes;
mod simulation_settings;
mod util;

use crate::distribution::distribute_mapping_weight;
use crate::nodes::{Message, Result, Work};
use crate::simulation_settings::c_1_simulation;
use algo::decode;
use std::fs::File;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn main() {
    c_1_simulation(6);
    // let file = File::open("/home/lu/CLionProjects/Split_learning_microcontrollers/Algorithms/json_files/test_17_63.json").expect("Failed to open file");
    // let layers = decode::decode_json(file);
    // distribute_mapping_weight(layers,6,(3,224,224),"./Simu".to_string());
}

