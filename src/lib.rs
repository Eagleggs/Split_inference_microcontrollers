use std::fmt::Debug;
use serde::{Serialize,Deserialize};
#[derive(Debug, Serialize, Deserialize)]
pub enum LayerWrapper {
    Convolution(Conv),
    Linear(Linear),
}

pub trait Layer {
    fn identify(&self) -> &str;
    fn get_weight(&self, position: Vec<i16>) -> f64;
    fn get_info(&self) -> &dyn Debug;
    fn get_bias(&self, p: i16) -> f64;
    fn get_all(&self) -> &dyn Debug;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conv {
    pub w: Vec<Vec<Vec<Vec<f64>>>>,
    pub info: ConvMapping,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConvMapping {
    pub o_pg: i16,
    pub i_pg: i16,
    pub s: (i16, i16),
    pub k: (i16, i16),
    pub i: (i16, i16, i16),
    pub o: (i16, i16, i16),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Linear {
    w: Vec<Vec<f64>>,
    info: LinearMapping,
    bias: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinearMapping {
    b_in: i16,
    c_in: i16,
    b_out: i16,
    c_out: i16,
}
pub trait IOMapping {
    fn map_to_input(){}
}
impl Layer for Conv {
    fn identify(&self) -> &str {
        "Convolution"
    }

    fn get_weight(&self, position: Vec<i16>) -> f64 {
        // Implement your logic to get the weight based on position
        // For example, you might want to access self.w with the given position
        assert_eq!(position.len(), 4);

        let r = (
            position[0].clone(),
            position[1].clone(),
            position[2].clone(),
            position[3].clone(),
        );

        // Directly index into the vector without cloning
        self.w[r.0 as usize][r.1 as usize][r.2 as usize][r.3 as usize]
    }

    fn get_info(&self) -> &dyn Debug {
        &self.info
    }

    fn get_bias(&self, i: i16) -> f64 {
        0.0
    }

    fn get_all(&self) -> &dyn Debug {
        self
    }
}


impl Layer for Linear {
    fn identify(&self) -> &str {
        "Linear"
    }

    fn get_weight(&self, position: Vec<i16>) -> f64 {
        // Implement your logic to get the weight based on position
        // For example, you might want to access self.w with the given position
        assert_eq!(position.len(), 2);
        let r = (position[0].clone() as usize, position[1].clone() as usize);
        return self.w[r.0][r.1];
    }

    fn get_info(&self) -> &dyn Debug {
        &self.info
    }

    fn get_bias(&self, p: i16) -> f64 {
        self.bias[p as usize]
    }

    fn get_all(&self) -> &dyn Debug {
        self
    }
}
