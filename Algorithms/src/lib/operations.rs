use crate::{InfoWrapper, Layer, WeightUnit};
use serde::{Deserialize, Serialize};

use std::cmp::max;
use std::ops::{BitAnd, BitOr};

pub fn distribute_weight(layer: &Box<dyn Layer>, total_cpu_count: i32) -> Vec<Vec<WeightUnit>> {
    let output_shape = layer.get_output_shape();
    let mut weight_to_send: Vec<Vec<WeightUnit>> = vec![Vec::new(); total_cpu_count as usize];
    let mut count: i32 = 0;
    let mut which_cpu = 0;
    let mut new_kernel_flag = false;
    let mut kernel_data: WeightUnit = WeightUnit {
        data: Vec::new(),
        which_kernel: 0,
        count: 0,
        start_pos_in: vec![],
        info: layer.get_info(),
    };
    match layer.get_info() {
        InfoWrapper::Convolution(_conv) => {
            let output_count: i32 = layer
                .get_output_shape()
                .into_iter()
                .product();
            let num_per_cpu: i32 = (output_count as f32 / total_cpu_count as f32).ceil() as i32;
            for j in 0..output_shape[0] {
                new_kernel_flag = true;
                for k in 0..output_shape[1] {
                    for m in 0..output_shape[2] {
                        let pos = layer.get_input(vec![j, k, m]);
                        if count / num_per_cpu != which_cpu {
                            weight_to_send[which_cpu as usize].push(kernel_data.clone());
                            rearrange_weight(&mut weight_to_send[which_cpu as usize]);
                            kernel_data.start_pos_in = pos[0].clone();
                            which_cpu += 1;
                            kernel_data.count = 0;
                        }
                        if new_kernel_flag {
                            if !kernel_data.data.is_empty() {
                                weight_to_send[which_cpu as usize].push(kernel_data.clone());
                            }
                            kernel_data.start_pos_in = pos[0].clone();
                            kernel_data.data = layer.get_weights_from_input(pos, j);
                            kernel_data.which_kernel = j as u16;
                            new_kernel_flag = false;
                            kernel_data.count = 0;
                        }
                        kernel_data.count += 1;
                        count += 1;
                    }
                }
            }

            weight_to_send[which_cpu as usize].push(kernel_data.clone());
            rearrange_weight(&mut weight_to_send[which_cpu as usize]);
        }
        InfoWrapper::Linear(info) => {
            let weight = layer.get_weights();
            let weight_shape = [info.c_in, info.c_out]; //1280,1000
            let col_per_cpu = (weight_shape[1] as f32 / total_cpu_count as f32).ceil() as i32;
            for j in 0..weight_shape[1] {
                for k in 0..weight_shape[0] {
                    kernel_data
                        .data
                        .push(weight[(k * weight_shape[0] + j) as usize]);
                }
                kernel_data.which_kernel = j as u16;
                which_cpu = j / col_per_cpu;
                weight_to_send[which_cpu as usize].push(kernel_data.clone());
                kernel_data.data.clear();
            }
        }
        InfoWrapper::ReLU6(_info) => {
            weight_to_send.resize(0, vec![]); // no weight data
        }
        InfoWrapper::BatchNorm2d(_info) => {
            //store in the coordinator, so size = 1
            weight_to_send.resize(1, vec![]);
            kernel_data.data = layer.get_weights();
            weight_to_send[0] = vec![kernel_data];
        }
    }
    weight_to_send
}
pub fn get_input_mapping(
    layer: &Box<dyn Layer>,
    total_cpu_count: i32,
    input_shape: Vec<usize>,
) -> Vec<Vec<Vec<u128>>> {
    let output_count: i32 = layer
        .get_output_shape()
        .into_iter()
        .product();
    let num_per_cpu: i32 = (output_count as f32 / total_cpu_count as f32).ceil() as i32;
    let mut mapping = vec![];
    match layer.get_info() {
        InfoWrapper::Convolution(conv) => {
            let mut kernel_size: (u16, u16) = (0, 0);
            kernel_size = (conv.k.0 as u16, conv.k.1 as u16);
            let padding_numbers = (kernel_size.0 / 2 * 2, kernel_size.1 / 2 * 2);
            mapping = vec![
                vec![
                    vec![0; input_shape[2] + padding_numbers.1 as usize];
                    input_shape[1] + padding_numbers.0 as usize
                ];
                input_shape[0]
            ]; //zero padding,kernel_size maximum = 3*3;
            let mut count: i32 = 0;
            let output_shape = layer.get_output_shape();
            let mut which_cpu = 0;
            for j in 0..output_shape[0] {
                for k in 0..output_shape[1] {
                    for m in 0..output_shape[2] {
                        if count / num_per_cpu != which_cpu {
                            which_cpu += 1;
                        }
                        let pos = layer.get_input(vec![j, k, m]);
                        //maximum 16 cpus,because of u16 type
                        let bit_coding: u128 = 1 << which_cpu;
                        for p in 0..pos.len() {
                            //-1 will be rounded to a very large value, so no need to check < 0
                            let a: usize = pos[p][0] as usize;
                            let b: usize = (pos[p][1] + (padding_numbers.0 / 2) as i32) as usize; // zero padding
                            let c: usize = (pos[p][2] + (padding_numbers.1 / 2) as i32) as usize;
                            mapping[a][b][c] = mapping[a][b][c].bitor(bit_coding);
                            if (b > input_shape[1] || b == 0) && padding_numbers.0 != 0
                                || (c > input_shape[2] || c == 0) && padding_numbers.1 != 0
                            {
                                mapping[a][b][c] = mapping[a][b][c].bitor(0b1 << 127);
                                // mark this as a padding position;
                            }
                        }
                        count += 1;
                    }
                }
            }
        }
        InfoWrapper::ReLU6(_info) => {}
        InfoWrapper::BatchNorm2d(_info) => {}
        InfoWrapper::Linear(_info) => {} // full pass
    }
    //empty mapping means full pass
    mapping
}
pub fn distribute_input(
    input: Vec<Vec<Vec<f32>>>,
    mapping: Vec<Vec<Vec<u128>>>,
    total_cpu_count: i32,
) -> Vec<Vec<f32>> {
    if mapping.is_empty() {
        return vec![];
    } //full pass
    let mut inputs_distribution = vec![Vec::new(); total_cpu_count as usize];
    let mut i_x = 0;
    let mut i_y = 0;
    for i in 0..mapping.len() {
        for j in 0..mapping[0].len() {
            for m in 0..mapping[0][0].len() {
                let map = mapping[i][j][m];
                if map == 0 {
                    continue;
                }
                let padding_flag = map >> 127 == 0b1;
                let mut cpu_mapped_to = Vec::new();
                for k in 0..127 {
                    if (map >> k).bitand(0b1) == 0b1 {
                        cpu_mapped_to.push(k);
                    }
                }
                for a in cpu_mapped_to {
                    if padding_flag {
                        inputs_distribution[a as usize].push(0.)
                    } else {
                        inputs_distribution[a as usize].push(input[i][i_y][i_x]);
                    }
                }
                if !padding_flag {
                    i_x += 1;
                    if i_x == input[0][0].len() {
                        i_x = 0;
                        i_y += 1;
                        if i_y == input[0].len() {
                            i_x = 0;
                            i_y = 0;
                        }
                    }
                }
            }
        }
    }
    inputs_distribution
}
pub fn distributed_computation(
    input_distribution: Vec<f32>,
    mut weight_distribution: Vec<WeightUnit>,
) -> Vec<f32> {
    let mut result = vec![Vec::new(); 10000];
    if weight_distribution.is_empty() {
        return vec![]
    }
    match &weight_distribution.clone()[0].info {
        InfoWrapper::Convolution(convMapping) => {
            let len = input_distribution.len();
            let mut start_point = 0;
            let mut max_visited = weight_distribution[0].start_pos_in.clone();
            let mut first_row = false;
            let mut out_side_rows = 0;
            let mut in_side_rows = 0;
            let mut completed_group = vec![];
            //analyse the weights to find the group that is completed
            let mut max_pos_count = 0;
            let mut prev_group = weight_distribution[0].which_kernel / convMapping.o_pg as u16;
            let mut offset = 0;
            let mut page_size = 0;
            let mut pages = vec![0; 10000];
            for i in 0..weight_distribution.len() {
                let padded_row = weight_distribution[i].start_pos_in[1] + convMapping.k.0 / 2;
                let padded_col = weight_distribution[i].start_pos_in[2] + convMapping.k.1 / 2;
                let cur_group = (weight_distribution[i].start_pos_in[0] / convMapping.i_pg) as u16;
                if prev_group != cur_group {
                    max_pos_count = 0;
                }
                let cur_pos_count =
                    padded_row / convMapping.s.1 * convMapping.o.2 + padded_col / convMapping.s.0;
                if cur_pos_count <= max_pos_count {
                    max_pos_count =
                        max(max_pos_count, cur_pos_count + weight_distribution[i].count);
                }
                if max_pos_count >= convMapping.o.1 * convMapping.o.2
                    && !completed_group.contains(&cur_group)
                {
                    completed_group.push(cur_group);
                }
                prev_group = cur_group;
            }
            for i in 0..weight_distribution.len() {
                let cur_group = weight_distribution[i].which_kernel / convMapping.o_pg as u16;
                if !completed_group.contains(&cur_group) && pages[cur_group as usize] == 0 {
                    pages[cur_group as usize] = get_input_count(&weight_distribution[i]);
                    if i + 1 < weight_distribution.len()
                        && weight_distribution[i + 1].which_kernel / convMapping.o_pg as u16
                            == cur_group
                    {
                        pages[cur_group as usize] += get_input_count(&weight_distribution[i + 1]);
                    }
                }
            }
            //do calculation
            prev_group = weight_distribution[0].which_kernel / convMapping.o_pg as u16;
            for i in 0..weight_distribution.len() {
                let mut padded_row = weight_distribution[i].start_pos_in[1] + convMapping.k.0 / 2;
                let mut padded_col = weight_distribution[i].start_pos_in[2] + convMapping.k.1 / 2;

                let mut adjustment = 0;
                if weight_distribution[i].count == 0 {
                    continue;
                }
                let group_nr = weight_distribution[i].which_kernel / convMapping.o_pg as u16;
                if prev_group != group_nr {
                    offset += page_size * convMapping.i_pg;
                    prev_group = group_nr;
                }
                if completed_group.contains(&group_nr) {
                    page_size = convMapping.i.1 * convMapping.i.2;
                } else {
                    page_size = pages[group_nr as usize];
                    if weight_distribution.len() == 1 {
                        page_size = len as i32 / convMapping.i_pg;
                    }
                }
                //handel heads
                if !completed_group.contains(&group_nr) && weight_distribution.len() == 2
                    || i == 0
                {
                    first_row = true;
                    if convMapping.i.2 - padded_row <= convMapping.k.1 {
                        // assuming at least 2 rows can be stored
                        out_side_rows = convMapping.k.1;
                    } else {
                        out_side_rows = convMapping.s.1;
                    }

                    adjustment = padded_col;
                    in_side_rows = convMapping.k.1 - out_side_rows;
                }
                //switch page
                if weight_distribution[i].start_pos_in > max_visited {
                    //switch group
                    if weight_distribution[i].start_pos_in[0] / convMapping.i_pg
                        != max_visited[0] / convMapping.i_pg
                    {
                        let rows_to_move_down = convMapping.k.1 - convMapping.s.1; // the last calculation will always move down a stride
                        start_point = start_point
                            + rows_to_move_down * convMapping.i.2
                            + (convMapping.i_pg - 1) * page_size;
                    } else {
                        //switch page within same group(only 2 weight unit per cpu)
                        start_point = input_distribution.len() as i32 / convMapping.i_pg
                            - get_input_count(&weight_distribution[i]);
                        first_row = true;
                    }
                } else {
                    // change within same completed page
                    let prev_end_pos = &weight_distribution[i.saturating_sub(1)].start_pos_in;
                    let diff = weight_distribution[i]
                        .start_pos_in
                        .iter()
                        .zip(prev_end_pos.iter())
                        .map(|(x, y)| y - x)
                        .collect::<Vec<i32>>();
                    start_point = start_point - diff[1] * convMapping.i.2 - diff[2];
                }

                while weight_distribution[i].count > 0 {
                    padded_row = weight_distribution[i].start_pos_in[1] + convMapping.k.0 / 2;
                    padded_col = weight_distribution[i].start_pos_in[2] + convMapping.k.1 / 2;
                    // adjustment = padded_col;
                    let mut acc = 0.;
                    for c in 0..convMapping.i_pg {
                        let channel = c * page_size;
                        for j in 0..convMapping.k.0 {
                            let col = j * convMapping.i.2;
                            for k in 0..convMapping.k.1 {
                                let row = k;
                                let mut index = (channel + col + row + start_point) as usize;
                                let mut remaining =
                                    (page_size - start_point + offset) * convMapping.i_pg;
                                //special case when 2 weight unit within the same group
                                if i == 0
                                    && weight_distribution.len() == 2
                                    && weight_distribution[i + 1].which_kernel
                                        / convMapping.o_pg as u16
                                        == weight_distribution[i].which_kernel
                                            / convMapping.o_pg as u16
                                    && !completed_group.contains(
                                        &(weight_distribution[i].which_kernel
                                            / convMapping.o_pg as u16),
                                    )
                                {
                                    remaining = (page_size
                                        - get_input_count(&weight_distribution[1])
                                        - start_point)
                                        * convMapping.i_pg
                                }
                                let to_complete = (convMapping.k.1 * convMapping.i.2
                                    - padded_col)
                                    * convMapping.i_pg;
                                // if weight_distribution[i].start_pos_in[1] == convMapping.i.1 - convMapping.k.1  - 1 && first_row{
                                //     to_complete -= adjustment * (convMapping.k.1 - 1);
                                // }
                                // handel tails
                                //111111111
                                //111******
                                //111******
                                //111******
                                if remaining < to_complete && !first_row {
                                    if padded_row >= convMapping.s.1 {
                                        out_side_rows = convMapping.s.1;
                                    } else {
                                        out_side_rows = convMapping.k.1;
                                    }
                                    in_side_rows = convMapping.k.1 - out_side_rows; //can not fill the gap, handel this in the bracket
                                    let empty_pos = (to_complete - remaining)
                                        / out_side_rows
                                        / convMapping.i_pg;
                                    if j > in_side_rows {
                                        index -= (j - in_side_rows) as usize * empty_pos as usize
                                    }
                                }
                                // handel heads
                                //***11111
                                //***11111
                                //11111111
                                else if first_row && remaining >= to_complete {
                                    if j < out_side_rows {
                                        index -= j as usize * adjustment as usize
                                    } else {
                                        index -= (out_side_rows - 1) as usize * adjustment as usize
                                    }
                                } else if first_row && remaining < to_complete {
                                    out_side_rows = convMapping.k.0;
                                    in_side_rows = 0;
                                    let empty_pos = (to_complete - remaining)
                                        / out_side_rows
                                        / convMapping.i_pg;

                                    //111***
                                    //111***
                                    //111***
                                    if j > in_side_rows && adjustment == 0 {
                                        index -= (j - in_side_rows) as usize * empty_pos as usize
                                    }
                                    //***111
                                    //***111
                                    //***111
                                    if j < out_side_rows {
                                        index -= j as usize * adjustment as usize
                                    } else {
                                        index -= (out_side_rows - 1) as usize * adjustment as usize
                                    }
                                }
                                acc += &input_distribution[index]
                                    * &weight_distribution[i].data[(c
                                        * convMapping.k.0
                                        * convMapping.k.1
                                        + j * convMapping.k.1
                                        + k)
                                        as usize];
                            }
                        }
                    }

                    result[weight_distribution[i].which_kernel as usize].push(acc);
                    weight_distribution[i].start_pos_in[2] += convMapping.s.0;
                    start_point += convMapping.s.0;
                    //change a row
                    if weight_distribution[i].start_pos_in[2]
                        + convMapping.k.0 / 2
                        + convMapping.k.0
                        > convMapping.i.2
                    {
                        weight_distribution[i].start_pos_in[2] = 0 - convMapping.k.0 / 2; //zero padding
                        weight_distribution[i].start_pos_in[1] += convMapping.s.1;

                        start_point = start_point - convMapping.s.0
                            + convMapping.k.0
                            + ((convMapping.s.1 - 1) * convMapping.i.1); // move to next row, first move left to the last position calculated, then add kernel size, then move down
                        if first_row {
                            start_point -= (out_side_rows - 1) * adjustment;
                            first_row = false;
                        }
                    }
                    max_visited = max(max_visited, weight_distribution[i].start_pos_in.clone());
                    weight_distribution[i].count -= 1;
                }
            }
        }
        InfoWrapper::ReLU6(_info) => {
            result[0] = input_distribution
                .into_iter()
                .map(|x| x.clamp(0., 6.0))
                .collect::<Vec<f32>>();
        }
        InfoWrapper::Linear(_info) => {
            for w in weight_distribution {
                let p = w.which_kernel;
                let r = w
                    .data
                    .into_iter()
                    .zip(input_distribution.iter())
                    .fold(0.0, |acc, (x, y)| acc + x * y);
                result[p as usize].push(r);
            }
        }
        InfoWrapper::BatchNorm2d(_info) => {}
    };
    result.concat()
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Mapping {
    count: Vec<u32>,
    map: Vec<Vec<u8>>,          // from which node,to which node
    channel: Vec<u8>,           //used for batch norm
    padding_pos: Vec<Vec<u32>>, //padding counts, when reached, should give 0
}

pub fn analyse_mapping(
    raw_mapping: Vec<Vec<Vec<u128>>>,
    num_cpus_previous: u8,
    num_cpus_next: u8,
) -> Vec<Mapping> {
    let num_per_mcu = ((raw_mapping.len() * raw_mapping[0].len() * raw_mapping[0][0].len()) as f32
        / num_cpus_previous as f32)
        .ceil() as u32;
    let mut mappping = vec![
        Mapping {
            count: vec![0; 70],
            map: vec![Vec::new(); 70],
            channel: vec![num_cpus_next + 1; 70],
            padding_pos: vec![Vec::new(); 70],
        };
        num_cpus_previous.into()
    ];
    let channels = raw_mapping.len();
    let cols = raw_mapping[0].len();
    let rows = raw_mapping[0][0].len();
    let mut cur_phase = vec![0; num_cpus_previous.into()];
    for i in 0..channels {
        for j in 0..cols {
            for k in 0..rows {
                if raw_mapping[i][j][k] == 0 {
                    continue;
                }
                let cur_mcu = (i * cols * rows + j * cols + k) / num_per_mcu as usize;
                let mut mcu_next = Vec::new();
                let padding_pos = &raw_mapping[i][j][k] >> 127 == 0b1;
                for a in 0..num_cpus_next {
                    if (&raw_mapping[i][j][k] >> a).bitand(0b1) == 0b1 {
                        mcu_next.push(a);
                    }
                }
                if (mcu_next != mappping[cur_mcu].map[cur_phase[cur_mcu]]
                    || i as u8 != mappping[cur_mcu].channel[cur_phase[cur_mcu]])
                    && !mappping[cur_mcu].map[cur_phase[cur_mcu]].is_empty()
                {
                    cur_phase[cur_mcu] += 1;
                }
                mappping[cur_mcu].channel[cur_phase[cur_mcu]] = i as u8;
                mappping[cur_mcu].map[cur_phase[cur_mcu]] = mcu_next;
                mappping[cur_mcu].count[cur_phase[cur_mcu]] += 1;
                let temp = mappping[cur_mcu].count[cur_phase[cur_mcu]];
                if padding_pos {
                    mappping[cur_mcu].padding_pos[cur_phase[cur_mcu]].push(temp)
                }
            }
        }
    }
    //reduce the mapping
    for m in &mut mappping {
        m.padding_pos.retain(|X| !X.is_empty());
        m.count.retain(|&x| x != 0);
        m.map.retain(|x| !x.is_empty());
        m.channel.retain(|&x| x != num_cpus_next + 1);
    }
    mappping
}
pub fn rearrange_weight(weight: &mut Vec<WeightUnit>) {
    weight.sort_by(|x, y| x.start_pos_in.cmp(&y.start_pos_in));
}
pub fn get_input_count(weight: &WeightUnit) -> i32 {
    if let InfoWrapper::Convolution(conv) = &weight.info {
        let rows = weight.count / conv.o.2;
        let col = weight.count - rows * conv.o.2;
        let mut in_rows = conv.k.1 + (rows - 1) * conv.s.1;
        let mut remain = conv.k.1 * conv.s.1 + (col - 1) * conv.s.0 * conv.s.1;
        if rows == 0 {
            in_rows = 0;
            remain +=
                (conv.k.1 - conv.s.1) * conv.k.0 + (col - 1) * (conv.k.1 - conv.s.1) * conv.s.0
        }
        if col == 0 {
            remain = 0;
        }
        
        // if weight.start_pos_in[2] != -1 {
        //     area += (conv.k.0 - conv.s.0) * conv.s.0;
        // }
        in_rows * conv.i.2 + remain
    } else {
        -1
    }
}
pub fn find_pagesize(page_vec: &Vec<(u16, i32)>, group_nr: u16) -> i32 {
    for x in page_vec {
        if x.0 == group_nr {
            return x.1;
        }
    }
    -1
}
