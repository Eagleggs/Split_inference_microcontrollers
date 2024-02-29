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
    let file = File::open(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Fused\fused_layers_141.json").expect("Failed to open file");
    let layers = decode::decode_json(file);
    distribute_mapping_weight(layers,8,(3,224,224),"./Simu".to_string());
    // c_1_simulation(4);
}
