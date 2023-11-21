use crate::lib::ConvMapping;

//RGB image
pub fn do_convolution(inputs: Vec<Vec<Vec<f64>>>, weights: Vec<Vec<Vec<f64>>>,bias:f64) -> f64 {
    let n = inputs.len();
    let col = inputs[0].len();
    let rol = inputs[0][0].len();
    assert_eq!(inputs.len(),weights.len());
    assert_eq!(inputs[0].len(),weights[0].len());
    assert_eq!(inputs[0][0].len(),weights[0][0].len());
    let mut result = 0.;
    for i in 0..n{
        for j in 0..col{
            for m in 0..rol{
                result += &inputs[i][j][m] * &weights[i][j][m];
            }
        }
    }
    result + bias
}
pub fn map_to_input(o_position: (i16, i16, i16), info: ConvMapping) -> Vec<Vec<i16>> {
    let h_offset = o_position.1 * info.s.0;
    let w_offset = o_position.2 * info.s.1;
    let which_group = (o_position.0 / info.o_pg) * info.i_pg;
    let mut result: Vec<Vec<i16>> = Vec::new();
    for q in 0..info.i_pg {
        for h in -&info.k.0 / 2..=&info.k.0 / 2 {
            for w in -&info.k.1 / 2..&info.k.1 / 2 {
                result.push(vec!(&which_group + &q, &h_offset + &h, &w_offset + w));
            }
        }
    }
    result
}
