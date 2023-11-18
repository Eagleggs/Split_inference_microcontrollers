use serde_json::{Result, Value};
use std::fs::File;
use std::io::Read;

pub fn json_to_weights(file:File)->Result<()>{
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the JSON string
    let restored_list: Vec<String> = serde_json::from_str(&contents)?;

    // Print the restored list
    println!("{:?}", restored_list);

    Ok(())
}
#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_read(){

    }
}