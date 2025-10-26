use rodio::{Decoder, OutputStream, Sink};
use std::{
    fs::File,
    io::BufReader,
    sync::{mpsc, Mutex},
    thread,
    time::Duration,
};

use crate::success;

#[derive(Debug)]
pub enum PlayerCommand {
    Pause,
    Resume,
    Stop,
}

pub struct AudioPlayer {
    command_sender: Mutex<mpsc::Sender<PlayerCommand>>,
    handle: Mutex<thread::JoinHandle<()>>,
}

impl AudioPlayer {

    pub fn new() -> Self {
        // Placeholder initialization; actual values will be set in play()
        let (tx, _rx) = mpsc::channel::<PlayerCommand>();
        let handle = thread::spawn(|| {});

        Self {
            command_sender: Mutex::new(tx),
            handle: Mutex::new(handle),
        }
    }

    pub fn play(& self, file_path: &str) {
        
        // 创建主线程与播放线程通信的通道
        let (tx, rx) = mpsc::channel::<PlayerCommand>();
        *self.command_sender.lock().unwrap() = tx;

        let file_path = file_path.to_string();
        // 启动播放线程
        *self.handle.lock().unwrap() = thread::spawn(move || {
            // 初始化 rodio 输出流
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            // 打开音频文件
            let file = BufReader::new(File::open(file_path).unwrap());
            let source = Decoder::new(file).unwrap();

            // 加载到 sink
            sink.append(source);
            sink.play();

            println!("[播放线程] 开始播放...");

            // 播放控制循环
            loop {
                // 使用非阻塞接收，防止暂停时破坏连续播放
                if let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        PlayerCommand::Pause => {
                            println!("[播放线程] 暂停播放");
                            sink.pause();
                        }
                        PlayerCommand::Resume => {
                            println!("[播放线程] 继续播放");
                            sink.play();
                        }
                        PlayerCommand::Stop => {
                            println!("[播放线程] 停止播放");
                            break;
                        }
                    }
                }

                if sink.empty() {
                    println!("[播放线程] 播放结束，自动退出");
                    break;
                }


                // 避免占用 CPU
                thread::sleep(Duration::from_millis(100));
            }

            // 结束播放
            sink.stop();
        });
        // self.handle = Arc::new(handle);
    }

    
    pub fn pause(&self) {
        success::lock_send(&self.command_sender, PlayerCommand::Pause);
    }

    pub fn resume(&self) {
        success::lock_send(&self.command_sender, PlayerCommand::Resume);
    }
    pub fn stop(& self) {
        success::lock_send(&self.command_sender, PlayerCommand::Stop);
    }
}