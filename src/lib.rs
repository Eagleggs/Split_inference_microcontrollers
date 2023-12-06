use serde::{Deserialize, Serialize};

use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize)]
pub enum LayerWrapper {
    Convolution(Conv),
    Linear(Linear),
    BatchNorm2d(Batchnorm2d),
    ReLU6(Relu6),
}

pub trait Layer {
    fn identify(&self) -> &str;
    fn get_input(&self, position: Vec<i16>) -> Vec<Vec<i16>>;
    // fn get_weight(&self,position:Vec<i16>) -> f64;
    fn get_output_shape(&self) -> Vec<i16>;
    fn get_info(&self) -> &dyn Debug;
    fn get_bias(&self, p: i16) -> f64;
    fn get_all(&self) -> &dyn Debug;
    fn print_weights_shape(&self);
    fn get_weights_from_input(&self, input: Vec<Vec<i16>>, c: i16) -> Vec<f64>;
    fn functional_forward(
        &self,
        input: &mut Vec<Vec<Vec<f64>>>,
    ) -> Result<&'static str, &'static str>;
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Batchnorm2d {
    w: Vec<f64>,
    bias: Vec<f64>,
    r_m: Vec<f64>,
    r_v: Vec<f64>,
    input_shape: Vec<i16>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Relu6 {
    input_shape: Vec<i16>,
}
pub trait IOMapping {
    fn map_to_input(&self, o_position: Vec<i16>) -> Vec<Vec<i16>>;
}

impl IOMapping for ConvMapping {
    fn map_to_input(&self, o_position: Vec<i16>) -> Vec<Vec<i16>> {
        assert_eq!(o_position.len(), 3);
        let h_offset = &o_position[1] * &self.s.0;
        let w_offset = &o_position[2] * &self.s.1;
        let which_group = (&o_position[0] / &self.o_pg) * &self.i_pg;
        let mut result: Vec<Vec<i16>> = Vec::new();
        for q in 0..self.i_pg {
            for h in -self.k.0 / 2..=self.k.0 / 2 {
                for w in -self.k.1 / 2..=self.k.1 / 2 {
                    result.push(vec![&which_group + &q, &h_offset + &h, &w_offset + w]);
                }
            }
        }
        result
    }
}

impl IOMapping for LinearMapping {
    fn map_to_input(&self, o_position: Vec<i16>) -> Vec<Vec<i16>> {
        assert_eq!(o_position.len(), 2);
        let mut result: Vec<Vec<i16>> = Vec::new();
        for i in 0..self.c_in {
            result.push(vec![o_position[0], i]);
        }
        result
    }
}

impl Layer for Conv {
    fn identify(&self) -> &str {
        "Convolution"
    }

    // fn get_weight(&self, position: Vec<i16>) -> f64 {
    //     // Implement your logic to get the weight based on position
    //     // For example, you might want to access self.w with the given position
    //     assert_eq!(position.len(), 4);
    //
    //     let r = (position[0], position[1], position[2], position[3]);
    //
    //     // Directly index into the vector without cloning
    //     self.w[r.0 as usize][r.1 as usize][r.2 as usize][r.3 as usize]
    // }

    fn get_input(&self, position: Vec<i16>) -> Vec<Vec<i16>> {
        self.info.map_to_input(position)
    }

    fn get_output_shape(&self) -> Vec<i16> {
        let mut reuslt = Vec::new();
        reuslt.push(self.info.o.0);
        reuslt.push(self.info.o.1);
        reuslt.push(self.info.o.2);
        reuslt
    }

    fn get_info(&self) -> &dyn Debug {
        &self.info
    }

    fn get_bias(&self, _i: i16) -> f64 {
        0.0
    }

    fn get_all(&self) -> &dyn Debug {
        self
    }

    fn print_weights_shape(&self) {
        println!(
            "weight shape:{:?},{:?},{:?},{:?}",
            self.w.len(),
            self.w[0].len(),
            self.w[0][0].len(),
            self.w[0][0][0].len()
        );
    }

    fn get_weights_from_input(&self, input: Vec<Vec<i16>>, output_channel: i16) -> Vec<f64> {
        let mut result = Vec::new();
        for i in 0..input.len() {
            let col = i % self.info.k.1 as usize;
            let row = (i / self.info.k.1 as usize) % self.info.k.0 as usize;
            let c = input[i][0] % self.info.i_pg;
            result.push(self.w[output_channel as usize][c as usize][row][col]);
        }
        result
    }

    fn functional_forward(
        &self,
        _input: &mut Vec<Vec<Vec<f64>>>,
    ) -> Result<&'static str, &'static str> {
        Err("This is a convolutional layer, not a functional layer")
    }
}

impl Layer for Linear {
    fn identify(&self) -> &str {
        "Linear"
    }

    // fn get_weight(&self, position: Vec<i16>) -> f64 {
    //     // Implement your logic to get the weight based on position
    //     // For example, you might want to access self.w with the given position
    //     assert_eq!(position.len(), 2);
    //     let r = (position[0] as usize, position[1] as usize);
    //     self.w[r.0][r.1]
    // }

    fn get_input(&self, position: Vec<i16>) -> Vec<Vec<i16>> {
        self.info.map_to_input(position)
    }

    fn get_output_shape(&self) -> Vec<i16> {
        let mut reuslt = Vec::new();
        reuslt.push(self.info.b_out);
        reuslt.push(self.info.c_out);
        reuslt
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

    fn print_weights_shape(&self) {
        println!("Weight shape:{:?},{:?}", self.w.len(), self.w[0].len());
    }

    fn get_weights_from_input(&self, input: Vec<Vec<i16>>, p: i16) -> Vec<f64> {
        let mut result: Vec<f64> = Vec::new();
        for i in 0..input.len() {
            result.push(self.w[p as usize][input[i][1] as usize]);
        }
        result
    }

    fn functional_forward(
        &self,
        _input: &mut Vec<Vec<Vec<f64>>>,
    ) -> Result<&'static str, &'static str> {
        Err("This is a Linear layer, not a functional layer")
    }
}

impl Layer for Batchnorm2d {
    fn identify(&self) -> &str {
        "Batchnorm2d"
    }

    fn get_input(&self, position: Vec<i16>) -> Vec<Vec<i16>> {
        vec![position]
    }

    fn get_output_shape(&self) -> Vec<i16> {
        self.input_shape.clone()
    }

    fn get_info(&self) -> &dyn Debug {
        &self.input_shape as &dyn Debug
    }

    fn get_bias(&self, p: i16) -> f64 {
        self.bias[p as usize]
    }

    fn get_all(&self) -> &dyn Debug {
        self
    }

    fn print_weights_shape(&self) {
        println!("Input shpae : {:?}", self.input_shape)
    }
    //assuming the input starts with channel, ie (c,h,w)
    fn get_weights_from_input(&self, input: Vec<Vec<i16>>, c: i16) -> Vec<f64> {
        let mut result = Vec::new();
        for _i in 0..input.len() {
            result.push(self.r_m[c as usize]);
            result.push(self.r_v[c as usize]);
            result.push(self.w[c as usize]);
            result.push(self.bias[c as usize]);
        }
        result
    }

    fn functional_forward(&self, input: &mut Vec<Vec<Vec<f64>>>) -> Result<&'static str, &'static str> {
        let c = input.len();
        let h = input[0].len();
        let w = input[0][0].len();
        for i in 0..c {
            // Update elements using batch normalization
            for j in 0..h {
                for k in 0..w {
                    input[i][j][k] =
                        (input[i][j][k] - self.r_m[i]) / (self.r_v[i] + 1e-5).sqrt() * self.w[i] + self.bias[i];
                }
            }
        }

        Ok("finished")
    }


}

impl Layer for Relu6 {
    fn identify(&self) -> &str {
        "Relu6"
    }

    fn get_input(&self, position: Vec<i16>) -> Vec<Vec<i16>> {
        vec![position]
    }

    fn get_output_shape(&self) -> Vec<i16> {
        self.input_shape.clone()
    }

    fn get_info(&self) -> &dyn Debug {
        &self.input_shape as &dyn Debug
    }

    fn get_bias(&self, _p: i16) -> f64 {
        0.0
    }

    fn get_all(&self) -> &dyn Debug {
        self
    }

    fn print_weights_shape(&self) {
        println!("Input shape: {:?}", self.input_shape)
    }

    fn get_weights_from_input(&self, _input: Vec<Vec<i16>>, _c: i16) -> Vec<f64> {
        vec![0.0]
    }

    fn functional_forward(
        &self,
        input: &mut Vec<Vec<Vec<f64>>>,
    ) -> Result<&'static str, &'static str> {
        for i in 0..input.len() {
            for j in 0..input[0].len() {
                for k in 0..input[0][0].len() {
                    if input[i][j][k] < 0.0 {
                        input[i][j][k] = 0.;
                    } else if input[i][j][k] >= 6.0 {
                        input[i][j][k] = 6.0;
                    }
                }
            }
        }
        Ok("finished")
    }
}
