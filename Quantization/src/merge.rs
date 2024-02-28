use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::slice::SliceIndex;
use std::task::ready;
use algo::{Conv, ConvMapping, Layer, LayerWrapper, Relu6};
use algo::InfoWrapper::Convolution;
use algo::LayerWrapper::Linear;
use std::io::Write;
//fuse batchnorm with convolution https://nenadmarkus.com/p/fusing-batchnorm-and-conv/
pub fn merge_batchnorm(layers:HashMap<i32,Box<dyn Layer>>){
    let mut modified_mapping : HashMap<i32,LayerWrapper> = HashMap::new();
    let mut prev_nr = 1;
    for a in 1..=layers.len(){
        let layer = layers.get(&(a as i32)).unwrap();
        if layer.identify() == "Convolution" {
            let next_layer = layers.get(&(a as i32 + 1)).unwrap();
            let Convolution(info) = layer.get_info_no_padding() else {panic!("impossible to decode convolution info from none convolution layer")};
            let i_pg = info.i_pg.clone();
            let kernel_shape = info.k.clone();
            if next_layer.identify() == "Batchnorm2d" {
                let mut fused_conv = Conv{
                    w: vec![],
                    info,
                    bias: vec![],
                };
                let mut conv_weights = layer.get_weights();
                let batch_norm = next_layer.get_weights();
                let num_kernels = batch_norm.len() / 4;
                let kernel_size = conv_weights.len() / num_kernels;
                assert_eq!(kernel_size as i32,kernel_shape.0 * kernel_shape.1 * i_pg);
                for i in 0..num_kernels{
                    let w_bn = batch_norm[num_kernels * 2 + i] / (batch_norm[num_kernels + i] + 1e-5).sqrt();
                    let bias_bn = batch_norm[num_kernels * 3 + i] - batch_norm[num_kernels * 2 + i] * batch_norm[i] / (batch_norm[num_kernels + i] + 1e-5).sqrt();
                    let bias_fused = w_bn * layer.get_bias(i as i32) + bias_bn;
                    let mut weights_fused = vec![vec![vec![0.;kernel_shape.0 as usize];kernel_shape.1 as usize];i_pg as usize];
                    for j in 0..kernel_size{
                        let dim = j / (kernel_shape.0 * kernel_shape.1) as usize;
                        let y = j % (kernel_shape.0 * kernel_shape.1) as usize / kernel_shape.0 as usize;
                        let x = j % (kernel_shape.0 * kernel_shape.1) as usize % kernel_shape.0 as usize;
                        weights_fused[dim][y][x] = ( w_bn * conv_weights[i * kernel_size + j]);
                    }
                    fused_conv.w.push(weights_fused);
                    fused_conv.bias.push(bias_fused);
                }
                modified_mapping.insert(prev_nr as i32,LayerWrapper::Convolution(fused_conv));
                prev_nr += 1;
            }
            else {
                let mut conv = Conv{
                    w: vec![],
                    info:info.clone(),
                    bias: vec![],
                };
                let num_kernels = info.o.0 as usize;
                let kernel_size = info.i_pg * kernel_shape.0 * kernel_shape.1;
                let conv_weights = layer.get_weights();
                for i in 0..num_kernels{
                    let mut w = vec![vec![vec![0.;kernel_shape.0 as usize];kernel_shape.1 as usize];i_pg as usize];
                    let bias = layer.get_bias(i as i32);
                    for j in 0..kernel_size as usize{
                        let dim = j / (kernel_shape.0 * kernel_shape.1) as usize;
                        let y = j % (kernel_shape.0 * kernel_shape.1) as usize / kernel_shape.0 as usize;
                        let x = j % (kernel_shape.0 * kernel_shape.1) as usize % kernel_shape.0 as usize;
                        w[dim][y][x] = conv_weights[i * kernel_size as usize + j];
                    }
                    conv.w.push(w);
                    conv.bias.push(bias);
                }
                modified_mapping.insert(prev_nr as i32,LayerWrapper::Convolution(conv));
                prev_nr += 1;
            }
        }
        else if layer.identify() == "Relu6" {
            let mut input_shape = layer.get_output_shape();
            input_shape.insert(0,1);
            let relu6 = Relu6{ input_shape };
            modified_mapping.insert(prev_nr as i32, LayerWrapper::ReLU6(relu6));
            prev_nr += 1;
        }
    }
    let serialized = serde_json::to_string(&modified_mapping).unwrap();
    let file_name = "fused_layers.json";
    let output_dir = "Fused";
    match fs::create_dir_all(&output_dir) {
        Ok(_) => println!("Folder created successfully"),
        Err(e) => eprintln!("Error creating folder: {}", e),
    }
    // Write the JSON string to a file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./".to_string() + &output_dir + "/" + &file_name)
        .unwrap();
    writeln!(file, "{}", serialized).unwrap();
}