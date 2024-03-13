extern crate core;

mod distribution;
mod nodes;
mod simulation_settings;
mod util;

use crate::distribution::{distribute_mapping_weight, distribute_mapping_weight_quant};
use crate::nodes::{Message, Result, Work};
use crate::simulation_settings::{c_1_simulation, c_1_simulation_quant};
use algo::decode;
use std::fs::File;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

fn main() {
    // let file = File::open(r"C:\Users\Lu JunYu\CLionProjects\Split_learning_microcontrollers_\Fused\fused_layers_16.json").expect("Failed to open file");
    // let layers = decode::decode_json(file);
    // distribute_mapping_weight_quant(layers,8,(3,224,224),"./Simu".to_string());
    c_1_simulation_quant(8);
}
