use std::io::empty;
use std::ops::BitAnd;
use serde::{Deserialize, Serialize};

//struct inside coordinator,moved to algorithms folder
// #[derive(Clone,Serialize,Deserialize)]
// pub struct Mapping{
//     count: Vec<u32>,
//     map:Vec<Vec<u8>>, // from which node,to which node
//     channel:Vec<u8>, //used for batch norm
//     padding_pos:Vec<Vec<u32>> //padding counts, when reached, should give 0
// }
//
// pub fn analyse_mapping(raw_mapping:Vec<Vec<Vec<u16>>>,num_cpus_previous:u8,num_cpus_next:u8)->Vec<Mapping>{
//     let num_per_mcu = ((raw_mapping.len() * raw_mapping[0].len() * raw_mapping[0][0].len()) as f32 / num_cpus_previous as f32).ceil() as u32;
//     let mut mappping = vec![Mapping{
//         count: vec![0;10],
//         map: vec![Vec::new();10],
//         channel: vec![0;10],
//         padding_pos: vec![Vec::new();10],
//     };num_cpus_previous.into()];
//     let channels = raw_mapping.len();
//     let cols = raw_mapping[0].len();
//     let rows = raw_mapping[0][0].len();
//     let mut cur_phase = vec![0;num_cpus_previous.into()];
//     for i in 0..channels{
//         for j in 0..cols{
//             for k in 0..rows{
//                 let cur_mcu = (i * cols * rows + j * cols + k)  / num_per_mcu as usize;
//                 let mut mcu_next = Vec::new();
//                 let padding_pos = &raw_mapping[i][j][k] >> 15 == 0b1;
//                 for a in 0..num_cpus_next{
//                     if (&raw_mapping[i][j][k] >> a).bitand(0b1) == 0b1{
//                         mcu_next.push(a);
//                     }
//                 }
//                 if (mcu_next != mappping[cur_mcu].map[cur_phase[cur_mcu]] || i as u8 != mappping[cur_mcu].channel[cur_phase[cur_mcu]])
//                 && !mappping[cur_mcu].map[cur_phase[cur_mcu]].is_empty() {
//                     cur_phase[cur_mcu] += 1;
//                 }
//                 mappping[cur_mcu].channel[cur_phase[cur_mcu]] = i as u8;
//                 mappping[cur_mcu].map[cur_phase[cur_mcu]] = mcu_next;
//                 mappping[cur_mcu].count[cur_phase[cur_mcu]] += 1;
//                 let temp = mappping[cur_mcu].count[cur_phase[cur_mcu]];
//                 if padding_pos{mappping[cur_mcu].padding_pos[cur_phase[cur_mcu]].push(temp)}
//             }
//         }
//     }
//     mappping
// }