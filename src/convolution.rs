use crate::decode::ConvMapping;

//RGB image
pub fn convolute(inputs: Vec<Vec<Vec<f64>>>, weights: Vec<Vec<Vec<f64>>>) -> f64 {
    todo!()
}
pub fn map_to_input(o_position: (i16, i16, i16), info: ConvMapping) -> Vec<(i16, i16, i16)> {
    let (c_in, h_in, w_in) = info.i;
    let (c_out, h_out, w_out) = info.o;
    let h_offset = o_position.1 * info.s.0;
    let w_offset = o_position.2 * info.s.1;
    let which_group = (o_position.0 / info.o_pg) * info.i_pg;
    let mut result: Vec<(i16, i16, i16)> = Vec::new();
    for q in -&info.i_pg / 2..&info.i_pg / 2 {
        for h in -&info.k.0 / 2..=&info.k.0 / 2 {
            for w in 0..=info.k.1 {
                result.push((&which_group + &q, &h_offset + &h, &w_offset + w));
            }
        }
    }
    result
}
