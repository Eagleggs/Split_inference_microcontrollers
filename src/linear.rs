pub fn do_linear(inputs: Vec<f64>, weights: Vec<f64>, bias: f64) -> f64 {
    let mut result = 0.;
    assert_eq!(inputs.len(), weights.len());
    for i in 0..inputs.len() {
        result += &weights[i] * &inputs[i];
    }
    result + bias
}
