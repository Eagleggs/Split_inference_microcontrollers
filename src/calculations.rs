use crate::lib::ConvMapping;

//assuming three dimensions
pub fn vector_mul_b(inputs: Vec<f64>, weights: Vec<f64>, bias: f64) -> f64 {
    let n = inputs.len();
    assert_eq!(inputs.len(), weights.len());
    let mut result = 0.;
    for i in 0..n {
        result += &inputs[i] * &weights[i];
    }
    result + bias
}
