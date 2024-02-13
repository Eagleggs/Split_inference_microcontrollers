pub fn sample_input_from_p_zero_padding(p: Vec<Vec<i32>>, input: &Vec<Vec<Vec<f32>>>) -> Vec<f32> {
    let mut result = Vec::new();
    for i in 0..p.len() {
        let a = &p[i];
        if a[0] < 0
            || a[1] < 0
            || a[2] < 0
            || a[0] >= input.len() as i32
            || a[1] >= input[0].len() as i32
            || a[2] >= input[0][0].len() as i32
        {
            result.push(0.);
        } else {
            result.push(input[a[0] as usize][a[1] as usize][a[2] as usize]);
        }
    }
    result
}
pub fn sample_input_linear(p: Vec<Vec<i32>>, input: &Vec<Vec<f32>>) -> Vec<f32> {
    let mut result = Vec::new();
    for i in 0..p.len() {
        let a = &p[i];
        result.push(input[a[0] as usize][a[1] as usize]);
    }
    result
}
pub fn split_u128_to_u8(number: u128) -> Vec<u8> {
    let mut result = Vec::new();

    // Iterate over each 8-bit chunk
    for i in 0..16 {
        let shift = i * 8;
        let chunk = ((number >> shift) & 0xFF) as u8;
        result.push(chunk);
    }

    result
}