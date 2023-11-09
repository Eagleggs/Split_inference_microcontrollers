use anyhow::{bail, Result};
use tch::Kind::{Double, Float, Uint8};
use tch::{
    nn::VarStore,
    vision::{imagenet, resnet::resnet18},
    Device, Kind,
};

pub fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let (model_file, image_file) = match args.as_slice() {
        [_, m, i] => (m.to_owned(), i.to_owned()),
        _ => bail!("usage: main model.pt image.jpg"),
    };
    let image = imagenet::load_image_and_resize(image_file, 164, 164)?;
    let mut vs = tch::nn::VarStore::new(Device::cuda_if_available());
    vs.load(model_file.clone())?;
    vs.
    let model = tch::CModule::load(model_file)?;
    let output = model.forward_ts(&[image.unsqueeze(0)])?.softmax(-1, Float);
    for (probability, class) in imagenet::top(&output, 5).iter() {
        println!("{:50} {:5.2}%", class, 100.0 * probability)
    }
    Ok(())
}
