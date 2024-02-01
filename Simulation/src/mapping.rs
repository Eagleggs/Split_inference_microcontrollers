//struct inside coordinator
pub struct Mapping{
    count: u32,
    map:(u8,u8), // from which node,to which node
    channel:u8, //used for batch norm
    padding_pos:Vec<u32> //padding counts, when reached, should give 0
}

pub fn analyse_mapping(raw_mapping:Vec<Vec<Vec<u16>>>)->Vec<Mapping>{
    todo!()
}