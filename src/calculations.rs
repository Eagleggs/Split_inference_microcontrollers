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
pub fn normalize(
    input: &mut Vec<f64>,
    data:Vec<f64>
) {
    assert_eq!(4 * input.len(), data.len());
    for i in 0..input.len() {
        input[i] = (input[i] - data[i * 4]) / (data[i * 4 + 1] + 1e-6).sqrt() * data[i * 4 + 2]
            + data[i * 4 + 3];
    }
}
