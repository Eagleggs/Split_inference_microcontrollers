use crate::util::{coordinator_send, decode_u128};
use algo::operations::Mapping;
use algo::WeightUnit;
use std::result;
use std::sync::mpsc;

pub struct Coordinator {
    mapping: Vec<Mapping>,
    batch_norm: Vec<f32>,
}
pub struct Worker {
    weights: Vec<WeightUnit>,
    inputs: Vec<f32>,
}

impl Coordinator {
    fn receive_and_send(
        &mut self,
        rec: &Vec<mpsc::Receiver<Option<f32>>>,
        send: &Vec<mpsc::Sender<Option<f32>>>,
    ) {
        for i in 0..rec.len() {
            let mut cur_phase = 0;
            let mut count = 0;
            loop {
                if count == self.mapping[i].padding_pos[cur_phase][0] {
                    let mut next_mcus = decode_u128(&self.mapping[i].map[cur_phase]);
                    coordinator_send(
                        next_mcus,
                        send,
                        0.,
                        &self.mapping[i].end_pos,
                        cur_phase,
                        count,
                    );
                    self.mapping[i].padding_pos[cur_phase].remove(0);
                    count += 1;
                    if count > self.mapping[i].count[cur_phase] {
                        cur_phase += 1;
                        count = 0;
                        if cur_phase >= self.mapping[i].count.len() {
                            // send to the next coordinator
                            todo!()
                        }
                    }
                } else if let Ok(data) = rec[i].recv() {
                    match data {
                        Some(d) => {
                            if count > self.mapping[i].count[cur_phase] {
                                cur_phase += 1;
                                count = 0;
                                if cur_phase >= self.mapping[i].count.len() {
                                    // send to the next coordinator
                                    todo!()
                                }
                            }
                            let channel = self.mapping[i].channel[cur_phase];
                            let norm = self.normalize(d, channel);
                            let mut next_mcus = decode_u128(&self.mapping[i].map[cur_phase]);
                            coordinator_send(
                                next_mcus,
                                send,
                                norm,
                                &self.mapping[i].end_pos,
                                cur_phase,
                                count,
                            );
                            count += 1;
                        }
                        None => {
                            break;
                        }
                    }
                }
            }
        }
    }
    fn normalize(&mut self, input: f32, channel: u8) -> f32 {
        todo!()
    }
}
impl Worker {
    fn receive(&mut self, rec: mpsc::Receiver<Option<f32>>) {
        loop {
            if let Ok(data) = rec.recv() {
                match data {
                    Some(d) => {
                        self.inputs.push(d);
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
    fn work(self, sender: mpsc::Sender<Option<f32>>) {
        let result = algo::operations::distributed_computation(self.inputs, self.weights);
        for i in result {
            sender.send(Some(i)).unwrap();
        }
        sender.send(None).expect("Send None is not allowed");
    }
}
