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
pub fn normalize(input:&mut Vec<f64>,mean:Vec<f64>,var:Vec<f64>,weight:Vec<f64>,bias:Vec<f64>){
    assert_eq!(mean.len(),var.len());
    assert_eq!(4 * input.len(),weight.len());
    for i in 0..input.len(){
        input[i] = (input[i] - mean[i * 4]) / (var[i * 4 + 1] + 1e-6).sqrt() * weight[i * 4 + 2] + bias[i * 4 + 3];
    }
}