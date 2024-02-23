use crate::util::{coordinator_send, decode_u128, wait_for_signal};
use algo::operations::Mapping;
use algo::WeightUnit;
use std::result;
use std::sync::mpsc;
use algo::calculations::batchnorm;
use serde::{Serialize,Deserialize};

pub type Work = Option<f32>;
pub type Result = Option<f32>;
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Work(Work),
    Result(Result),
    Quit,
    StartTransmission,
}
#[derive(Serialize, Deserialize)]
pub struct Coordinator {
    pub(crate) mapping: Vec<Mapping>,
    pub(crate) batch_norm: Vec<f32>,
    pub(crate) operations:Vec<u8>, //todo! add operations in Coordinator
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Worker {
    pub(crate) weights: Vec<WeightUnit>,
    pub(crate) inputs: Vec<f32>,
    pub status:bool,
}

impl Coordinator {
    pub fn receive_and_send(
        &mut self,
        rec: &mpsc::Receiver<Message>,
        send: &Vec<mpsc::Sender<Message>>,
        worker_swarm_size : u8,
    ) {
        for i in 0..worker_swarm_size as usize {
            send[i].send(Message::StartTransmission).expect("start transmission failed.");
            let mut cur_phase = 0;
            let mut count = 0;
            let mut total_count = 0;
            loop {
                if cur_phase < self.mapping[i].count.len() && !self.mapping[i].padding_pos[cur_phase].is_empty() && count == self.mapping[i].padding_pos[cur_phase][0] {
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
                        // println!("{:?},{:?}",count,i);
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
                            total_count += 1;
                            if count == self.mapping[i].count[cur_phase] {
                                cur_phase += 1;
                                count = 0;
                                if cur_phase >= self.mapping[i].count.len() {
                                    //todo! send to the next coordinator
                                    continue;
                                }
                            }
                        }
                        Message::Result(None) => {
                            // assert_eq!(count,0);
                            // println!("count:{:?},cur_phase_count:{:?}",count,self.mapping[i].count[cur_phase]);
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    fn normalize(&mut self, input: f32, channel: u8) -> f32 {
        let mut result = 0.;
        for op in &self.operations{
            match op {
                1 =>{
                    result = batchnorm(input, &self.batch_norm, channel);
                } //batchnorm
                2 =>{
                    result = result.clamp(0.,6.0);
                } //relu6
                _ =>{}
            }
        }
        result
    }
}
impl Worker {
    pub fn receive(&mut self, rec: &mpsc::Receiver<Message>) {
        loop {
            if let Ok(data) = rec.recv() {
                match data {
                    Message::Work(Some(d)) => {
                        self.inputs.push(d);
                    }
                    Message::Work(None) => {
                        println!("worker breaking");
                        break;
                    }
                    Message::Quit =>{self.status = false; break}
                    _ => {}
                }
            }
        }
    }
    pub fn work(self, sender: &mpsc::Sender<Message>,rec: &mpsc::Receiver<Message>) {
        let result = algo::operations::distributed_computation(self.inputs, self.weights);
        println!("coordinator finished calculation");
        wait_for_signal(rec);
        for i in result {
            sender.send(Message::Result(Some(i))).unwrap();
        }
        sender.send(Message::Result(None)).expect("Send None is not allowed");
    }

}
