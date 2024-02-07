use std::result;
use std::sync::mpsc;
use algo::operations::Mapping;
use algo::WeightUnit;

pub struct Coordinator{
    mapping:Vec<Mapping>,
    //todo
}
pub struct Worker{
    weights: Vec<WeightUnit>,
    inputs: Vec<f32>,
    //todo
}

impl Coordinator{
    //todo
}
impl Worker{
    fn receive(&mut self, rec : mpsc::Receiver<f32>){
        while let Ok(data) = rec.recv(){
            self.inputs.push(data);
        }
    }
    fn work(self){
        let result =algo::operations::distributed_computation(self.inputs,self.weights);
    }
}