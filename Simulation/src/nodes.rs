use algo::operations::Mapping;
use algo::WeightUnit;
use std::result;
use std::sync::mpsc;

pub struct Coordinator {
    mapping: Vec<Mapping>,
wor    batch_norm: Vec<f32>,
    //todo
}
pub struct Worker {
    weights: Vec<WeightUnit>,
    inputs: Vec<f32>,
    //todo
}

impl Coordinator {
    fn receive_and_send(&mut self, rec: Vec<mpsc::Receiver<f32>>,send_pipes:Vec<mpsc::Sender<f32>>){todo!()}
    fn normalize(&mut self,input: f32,channel:u16)->f32{todo!()}

    //todo
}
impl Worker {
    fn receive(&mut self, rec: mpsc::Receiver<f32>) {
        loop {
            if let Ok(data) = rec.recv() {
                if data == '*' {
                    break;
                }
                self.inputs.push(data);
            }
        }
    }
    fn work(self) -> Vec<f32> {
        let result = algo::operations::distributed_computation(self.inputs, self.weights);
        result
    }
}
