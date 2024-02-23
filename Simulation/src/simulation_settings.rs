use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use crate::nodes::Message;
use crate::{perform_work, util};
use crate::util::{decode_coordinator, decode_worker, flatten_3d_array, generate_test_input};

pub fn preparation_phase(){
    todo!()
} //distribute weight, analyse mapping,distribute coordinators,distribute workers write into files.
pub fn c_1_w60_simulation(){// 创建一个消息发送者和多个消息接收者

    let (coordinator_sender, coordinator_receiver) = mpsc::channel::<Message>();

    let mut handles = vec![];
    let mut worker_send_channel  = vec![];
    for worker_id in 0..60 {
        let (worker_sender, worker_receiver) = mpsc::channel::<Message>();
        let coordinator_sender_clone = coordinator_sender.clone();
        let file_name = format!("./Simu1/worker_{:?}.json",worker_id);
        let handle = thread::spawn(move || {
            let mut phase  = 0;
            // Worker线程的接收端
            loop{
                let mut worker = decode_worker(&file_name,phase).unwrap();
                worker.receive(&worker_receiver);
                if worker.status == false { break; }
                worker.work(&coordinator_sender_clone,&worker_receiver);
                phase += 1;
            }
        });

        // 主线程将Worker线程的发送端和句柄保存在Vec中
        handles.push( handle);
        worker_send_channel.push(worker_sender);
    }
    let file_name = "./Simu1/Coordinator.json";
    let coordinator_handle = thread::spawn(move ||{
        let mut phase = 0;
        loop{
            let mut coordinator = decode_coordinator(file_name,phase).unwrap();
            coordinator.receive_and_send(&coordinator_receiver,&worker_send_channel,60);
            phase += 1;
        }
    });
    handles.push(coordinator_handle);
    //intput
    let input = flatten_3d_array(generate_test_input(226,226,3));
    let num_per_cpu = ((226 * 226 * 3) as f32 / 60 as f32).ceil() as u32;
    //jump start the simulation
    for i in 0..60{
        let coordinator_sender_clone = coordinator_sender.clone();
        for j in 0..num_per_cpu{
            let index = (i * num_per_cpu + j) as usize;
            if index < input.len() {
                coordinator_sender_clone.send(Message::Result(Some(input[index]))).expect("Start failed");
            }
        }
        coordinator_sender_clone.send(Message::Result(None)).expect("start failed");
    }
    // 等待所有Worker线程完成
    for handle in handles {
        handle.join().unwrap();
    }


}//start the simulation
