use crate::nodes::Message;
use crate::util::{decode_coordinator, decode_worker, flatten_3d_array, generate_test_input};
use crate::{perform_work, util};
use chrono::prelude::*;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;
pub fn preparation_phase() {
    todo!()
} //distribute weight, analyse mapping,distribute coordinators,distribute workers write into files.
pub fn c_1_simulation(num_workers: u8) {
    // 创建一个消息发送者和多个消息接收者

    let (coordinator_sender, coordinator_receiver) = mpsc::channel::<Message>();
    let start_time = Instant::now();
    let mut handles = vec![];
    let mut worker_send_channel = vec![];
    for worker_id in 0..num_workers {
        let (worker_sender, worker_receiver) = mpsc::channel::<Message>();
        let coordinator_sender_clone = coordinator_sender.clone();
        let file_name = format!("./Simu/worker_{:?}.json", worker_id);
        let handle = thread::spawn(move || {
            let mut phase = 0;
            let mut buffer = Vec::new();
            // Worker线程的接收端
            loop {
                let mut worker = decode_worker(&file_name, phase, buffer).unwrap();
                println!(
                    "worker{:?} start receiving,time:{:?}",
                    worker_id,
                    start_time.elapsed()
                );
                worker.receive(&worker_receiver, worker_id);
                println!(
                    "worker{:?} finished receiving,time:{:?}",
                    worker_id,
                    start_time.elapsed()
                );
                if worker.status == true {
                    break;
                }
                buffer = worker.work(&coordinator_sender_clone, &worker_receiver, worker_id); //buffer is the data received while working
                phase += 1;
            }
            println!("worker{:?}, exited", worker_id);
        });

        // 主线程将Worker线程的发送端和句柄保存在Vec中
        handles.push(handle);
        worker_send_channel.push(worker_sender);
    }
    let file_name = "./Simu/Coordinator.json";
    let coordinator_handle = thread::spawn(move || {
        let mut phase = 0;
        loop {
            let mut coordinator = decode_coordinator(file_name, phase).unwrap();
            coordinator.receive_and_send(&coordinator_receiver, &worker_send_channel, num_workers);
            println!("phase{:?} finished", phase);
            phase += 1;
        }
    });
    handles.push(coordinator_handle);
    //intput
    let input = flatten_3d_array(generate_test_input(224, 224, 3));
    let num_per_cpu = ((224 * 224 * 3) as f32 / num_workers as f32).ceil() as u32;
    //jump start the simulation
    let mut count = 0;
    for i in 0..num_workers {
        let coordinator_sender_clone = coordinator_sender.clone();
        for j in 0..num_per_cpu {
            if count < input.len() {
                coordinator_sender_clone
                    .send(Message::Result(Some(input[count])))
                    .expect("Start failed");
            }
            count += 1;
        }
        coordinator_sender_clone
            .send(Message::Result(None))
            .expect("start failed");
    }
    // 等待所有Worker线程完成
    for handle in handles {
        handle.join().unwrap();
    }
} //start the simulation
