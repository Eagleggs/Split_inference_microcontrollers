use crate::lib::ConvMapping;

//assuming three dimensions
pub fn do_convolution(inputs: Vec<Vec<Vec<f64>>>, weights: Vec<Vec<Vec<f64>>>, bias: f64) -> f64 {
    let n = inputs.len();
    let col = inputs[0].len();
    let rol = inputs[0][0].len();
    assert_eq!(inputs.len(), weights.len());
    assert_eq!(inputs[0].len(), weights[0].len());
    assert_eq!(inputs[0][0].len(), weights[0][0].len());
    let mut result = 0.;
    for i in 0..n {
        for j in 0..col {
            for m in 0..rol {
                result += &inputs[i][j][m] * &weights[i][j][m];
            }
        }
    }
    result + bias
}
