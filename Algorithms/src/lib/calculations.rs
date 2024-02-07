//assuming three dimensions
pub fn vector_mul_b(inputs: Vec<f32>, weights: Vec<f32>, bias: f32) -> f32 {
    let n = inputs.len();
    assert_eq!(inputs.len(), weights.len());
    let mut result = 0.;
    for i in 0..n {
        result += &inputs[i] * &weights[i];
    }
    result + bias
}
pub fn _normalize(input: &mut Vec<f32>, data: Vec<f32>) {
    assert_eq!(4 * input.len(), data.len());
    for i in 0..input.len() {
        input[i] = (input[i] - data[i * 4]) / (data[i * 4 + 1] + 1e-6).sqrt() * data[i * 4 + 2]
            + data[i * 4 + 3];
    }
}
