use std::sync::mpsc;
use crate::nodes::Message;


pub fn decode_u128(input: &Vec<u8>) -> Vec<usize> {
    let mut next_mcus = Vec::new();
    let mut offset = 0;
    for t in input {
        for i in 0..8 {
            if (t >> i) & 0b1 == 0b1 {
                next_mcus.push(offset + i)
            }
        }
        offset += 8;
    }
    next_mcus
}
pub fn coordinator_send(
    next_mcus: Vec<usize>,
    send: &Vec<mpsc::Sender<Message>>,
    val: f32,
    end_pos: &Vec<(u16, u8, u32)>,
    cur_phase: usize,
    count: u32,
) {
    next_mcus.into_iter().for_each(|x| {
        send[x].send(Message::Work(Some(val))).expect("Coordinator send failed");
        for e in end_pos {
            if e.0 == cur_phase as u16 && e.1 == x as u8 && e.2 == count {
                send[x].send(Message::Work(None)).expect("Coordinator send none failed");
            }
        }
    });
}
pub fn wait_for_signal(rec: &mpsc::Receiver<Message>){
    loop{
        match rec.recv() {
            Ok(Message::StartTransmission) => { break;}
            _ => {}
        }
    }
}