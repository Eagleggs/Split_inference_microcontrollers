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
pub fn batchnorm(input: f32,data: &Vec<f32>,channel:u8)->f32{
    assert_eq!(data.len() % 4, 0);
    let size = data.len() / 4;
    (input - data[channel as usize]) / (data[size + channel as usize] + 1e-5).sqrt() * data[size * 2 + channel as usize] + data[size * 3 + channel as usize]
}
