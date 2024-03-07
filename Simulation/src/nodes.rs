use crate::util::{coordinator_send, decode_u128, send_to_all_workers, wait_for_signal};
use algo::calculations::batchnorm;
use algo::operations::Mapping;
use algo::WeightUnit;
use serde::{Deserialize, Serialize};
use std::result;
use std::sync::mpsc;
use std::sync::mpsc::RecvError;
use std::time::Instant;

pub type Work = Option<f32>;
pub type Result = Option<f32>;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Work(Work),
    Result(Result),
    Quit,
    StartTransmission,
}
#[derive(Serialize, Deserialize)]
pub struct Coordinator {
    pub(crate) mapping: Vec<Mapping>,
    // pub(crate) batch_norm: Vec<f32>,
    // pub(crate) operations: Vec<u8>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Worker {
    pub(crate) weights: Vec<WeightUnit>,
    pub(crate) inputs: Vec<f32>,
    pub status: bool,
    pub operations: Vec<u8>,
}

impl Coordinator {
    pub fn receive_and_send(
        &mut self,
        rec: &mpsc::Receiver<Message>,
        send: &Vec<mpsc::Sender<Message>>,
        worker_swarm_size: u8,
    ) {
        for i in 0..worker_swarm_size as usize {
            send[i]
                .send(Message::StartTransmission)
                .expect("start transmission failed.");
            println!("coordinator start receiving from {:?}", i);
            let mut cur_phase = 0;
            let mut count = 0;
            let mut total_count = 0;
            loop {
                if !self.mapping.is_empty()
                    && cur_phase < self.mapping[i].count.len()
                    && !self.mapping[i].padding_pos[cur_phase].is_empty()
                    && count == self.mapping[i].padding_pos[cur_phase][0]
                {
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
                    total_count += 1;
                    if count == self.mapping[i].count[cur_phase] {
                        // println!("coordinator receiving from {:?} switch phase,count{:?},phase:{:?},total_count:{:?}",i,count,cur_phase,total_count);
                        cur_phase += 1;
                        count = 0;
                        if cur_phase >= self.mapping[i].count.len() {
                            //todo! send to the next coordinator
                            continue;
                        }
                    }
                } else if let Ok(data) = rec.recv() {
                    // println!("received data from {:?},data{:?} ",i,data);
                    match data {
                        Message::Result(Some(d)) => {
                            if self.mapping.is_empty() {
                                send_to_all_workers(Message::Work(Some(d)), send);
                                continue;
                            }
                            let mut next_mcus = decode_u128(&self.mapping[i].map[cur_phase]);
                            coordinator_send(
                                next_mcus,
                                send,
                                d,
                                &self.mapping[i].end_pos,
                                cur_phase,
                                count,
                            );
                            count += 1;
                            total_count += 1;
                            if count == self.mapping[i].count[cur_phase] {
                                // println!("coordinator receiving from {:?} switch phase,count{:?},phase:{:?},total_count:{:?}",i,count,cur_phase,total_count);
                                cur_phase += 1;
                                count = 0;
                                if cur_phase >= self.mapping[i].count.len() {
                                    //todo! send to the next coordinator
                                    continue;
                                }
                            }
                        }
                        Message::Result(None) => {
                            if self.mapping.is_empty() && i as u8 == worker_swarm_size - 1 {
                                send_to_all_workers(Message::Work(None), send);
                            }
                            // assert_eq!(count, 0);
                            // assert_eq!(cur_phase, self.mapping[i].count.len());
                            // println!("finished receiving from {:?}",i);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    // fn normalize(&mut self, input: f32, channel: u8) -> f32 {
    //     let mut result : f32 = 0.;
    //     for op in &self.operations {
    //         match op {
    //             1 => {
    //                 // result = batchnorm(input, &self.batch_norm, channel);
    //             } //batchnorm
    //             2 => {
    //                 result = result.clamp(0., 6.0);
    //             } //relu6
    //             _ => {}
    //         }
    //     }
    //     result
    // }
    pub fn receive_and_terminate(
        &self,
        rec: &mpsc::Receiver<Message>,
        send: &Vec<mpsc::Sender<Message>>,
        worker_swarm_size: u8,
    ) -> Vec<f32> {
        let mut result_vec = Vec::new();
        println!("coordinator receiving result");
        for i in 0..worker_swarm_size as usize {
            send[i]
                .send(Message::StartTransmission)
                .expect("start transmission failed.");
            println!("coordinator start receiving from {:?}", i);
            loop {
                if let Ok(data) = rec.recv() {
                    match data {
                        Message::Result(Some(d)) => {
                            result_vec.push(d);
                        }
                        Message::Result(None) => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
            println!("coordinator send quit to {}", i);
            send[i].send(Message::Quit).unwrap();
        }
        result_vec
    }
}
impl Worker {
    pub fn receive(&mut self, rec: &mpsc::Receiver<Message>, id: u8) {
        loop {
            if let Ok(data) = rec.recv() {
                match data {
                    Message::Work(Some(d)) => {
                        self.inputs.push(d);
                    }
                    Message::Work(None) => {
                        println!("worker{:?} breaking", id);
                        break;
                    }
                    Message::Quit => {
                        self.status = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn work(
        self,
        sender: &mpsc::Sender<Message>,
        rec: &mpsc::Receiver<Message>,
        id: u8,
    ) -> Vec<f32> {
        let mut result = algo::operations::distributed_computation(self.inputs, self.weights);
        if self.operations.contains(&1) {
            for i in 0..result.len() {
                result[i] = result[i].clamp(0., 6.);
            }
        }
        let mut buffer = Vec::new();
        // println!("worker{:?},result size:{:?}",id,result.len());
        wait_for_signal(rec, &mut buffer);
        let start_time = Instant::now();
        for i in result {
            sender.send(Message::Result(Some(i))).unwrap();
        }
        sender
            .send(Message::Result(None))
            .expect("Send None is not allowed");
        println!(
            "worker{:?} send None,time consumed:{:?}",
            id,
            start_time.elapsed()
        );
        buffer
    }
    pub fn adaptive_pooling(&mut self) {
        // can be done in worker or coordinator, here I choose to do it in worker, may adjust this according to the ram size of the worker
        let window_size = self.inputs.len() / 1280;
        let len = self.inputs.len();
        let mut result_index = 0;
        while result_index + window_size <= self.inputs.len() {
            let avg = self.inputs[result_index..result_index + window_size]
                .iter()
                .sum::<f32>()
                / window_size as f32;
            self.inputs[result_index] = avg;
            self.inputs
                .drain(result_index + 1..result_index + window_size);
            result_index += 1;
        }
        assert_eq!(self.inputs.len(), 1280);
    }
}
