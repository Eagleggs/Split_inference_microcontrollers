use algo::operations::Mapping;
use algo::WeightUnit;
use std::result;
use std::sync::mpsc;

pub struct Coordinator {
    mapping: Vec<Mapping>,
    batch_norm: Vec<f32>,
    //todo
}
pub struct Worker {
    weights: Vec<WeightUnit>,
    inputs: Vec<f32>,
    //todo
}

impl Coordinator {
    fn receive_and_send(
        &mut self,
        rec: Vec<mpsc::Receiver<Option<f32>>>,
        send: Vec<mpsc::Sender<Option<f32>>>,
    ) {
        for i in 0..rec.len() {
            let mut cur_phase = 0;
            loop {
                if self.mapping[i].count[cur_phase] == self.mapping[i].padding_pos[cur_phase][0] {
                    let mut next_mcus = Vec::new();
                    let mut offset = 0;
                    for t in &self.mapping[i].map[cur_phase] {
                        for i in 0..8 {
                            if (t >> i) & 0b1 == 0b1 {
                                next_mcus.push(offset + i)
                            }
                        }
                        offset += 8;
                    }
                    next_mcus
                        .into_iter()
                        .for_each(|x| send[x].send(Some(0.)).expect("Coordinator send failed"));
                    self.mapping[i].padding_pos[cur_phase].remove(0);
                    self.mapping[i].count[cur_phase] -= 1;
                    if self.mapping[i].count[cur_phase] == 0 {
                        cur_phase += 1;
                        if cur_phase >= self.mapping[i].count.len() {
                            // send to the next coordinator
                            todo!()
                        }
                    }
                } else if let Ok(data) = rec[i].recv() {
                    match data {
                        Some(d) => {
                            if self.mapping[i].count[cur_phase] == 0 {
                                cur_phase += 1;
                                if cur_phase >= self.mapping[i].count.len() {
                                    // send to the next coordinator
                                    todo!()
                                }
                            }
                            let channel = self.mapping[i].channel[cur_phase];
                            let norm = self.normalize(d, channel);
                            let mut next_mcus = Vec::new();
                            let mut offset = 0;
                            for t in &self.mapping[i].map[cur_phase] {
                                for i in 0..8 {
                                    if (t >> i) & 0b1 == 0b1 {
                                        next_mcus.push(offset + i)
                                    }
                                }
                                offset += 8;
                            }
                            next_mcus.into_iter().for_each(|x| {
                                send[x].send(Some(norm)).expect("Coordinator send failed")
                            });
                            self.mapping[i].count[cur_phase] -= 1;
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

    //todo
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
                        //todo! need to modify the mapping to encode the end positions
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
