use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use crate::nodes::Message;
use crate::{perform_work, util};
use crate::util::{decode_coordinator, decode_worker};

pub fn preparation_phase(){
    todo!()
} //distribute weight, analyse mapping,distribute coordinators,distribute workers write into files.
pub fn c_1_w60_simulation(){// 创建一个消息发送者和多个消息接收者

    let coordinator = decode_coordinator("todo");
    let (coordinator_sender, coordinator_receiver) = mpsc::channel::<Message>();

    let mut worker_handles = vec![];
    for worker_id in 0..60 {
        let (worker_sender, worker_receiver) = mpsc::channel::<Message>();
        let coordinator_sender_clone = coordinator_sender.clone();

        let handle = thread::spawn(move || {
            // Worker线程的接收端
            loop{
                let mut worker = decode_worker("todo");
                worker.receive(&worker_receiver);
                if worker.status == false { break; }
                worker.work(&coordinator_sender_clone,&worker_receiver)
            }
        });

        // 主线程将Worker线程的发送端和句柄保存在Vec中
        worker_handles.push((worker_sender, handle));
    }


    // 等待所有Worker线程完成
    for (_, handle) in worker_handles {
        handle.join().unwrap();
    }

    // 主线程接收消息
    for _ in 0..60 {
        match coordinator_receiver.recv() {
            Ok(Message::Result(Some(result))) => {
                println!("Coordinator received result: {}", result);
                // 在这里可以处理结果，例如聚合结果或进行其他操作
            }
            Ok(Message::Quit) => break, // 终止主线程
            _ => {}
        }
    }
}//start the simulation
