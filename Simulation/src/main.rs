mod nodes;
mod util;
mod Phases;

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Work = Option<f32>;
type Result = Option<f32>;

enum Message {
    Work(Work),
    Result(Result),
    Quit,
}

fn main() {
    // 创建一个消息发送者和多个消息接收者
    let (coordinator_sender, coordinator_receiver) = mpsc::channel::<Message>();

    // 创建60个Worker线程
    let mut worker_handles = vec![];

    for worker_id in 0..60 {
        let (worker_sender, worker_receiver) = mpsc::channel::<Message>();
        let coordinator_sender_clone = coordinator_sender.clone();

        let handle = thread::spawn(move || {
            // Worker线程的接收端
            let worker_receiver = Arc::new(Mutex::new(worker_receiver));

            loop {
                // 在这里等待Coordinator发送工作或终止消息
                let message = worker_receiver.lock().unwrap().recv();
                match message {
                    Ok(Message::Work(Some(work))) => {
                        // 模拟工作的计算
                        let result = perform_work(Some(work));
                        // 将结果发送给Coordinator
                        coordinator_sender_clone.send(Message::Result(result)).unwrap();
                    }
                    Ok(Message::Quit) => break, // 终止线程
                    _ => {}
                }
            }
        });

        // 主线程将Worker线程的发送端和句柄保存在Vec中
        worker_handles.push((worker_sender, handle));
    }

    // 模拟Coordinator发放工作
    for id in 0..worker_handles.len() {
        let worker_sender = &worker_handles[id].0;
        let work = generate_work(id as f32);
        worker_sender.send(Message::Work(work)).unwrap();
    }

    // 关闭所有Worker线程
    for (worker_sender, _) in &worker_handles {
        worker_sender.send(Message::Quit).unwrap();
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
}

fn generate_work(id:f32) -> Work {
    // 模拟生成工作内容
    Some(id)
}

fn perform_work(work: Work) -> Result {
    // 模拟工作的计算
    work
}
