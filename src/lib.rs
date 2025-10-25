use std::time::{Duration, Instant};
use std::thread;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use rand::Rng;
use chrono::{DateTime, Local, Duration as ChronoDuration};
use anyhow::Result;
use std::sync::Mutex;


/// adfdf
/// fdfd
/// fsdfa
pub struct ClassTicker {
    running: Mutex<bool>,
    pub class_time: Mutex<u64>,    // 默认 90 分钟（5400秒）
    pub rest_time: Mutex<u64>,     // 默认休息 20 分钟（1200秒）
    pub elapsed_time: Mutex<u64>,
    start_time: Mutex<Instant>,
    end_time: Mutex<DateTime<Local>>,
}

impl ClassTicker {
    pub fn new() -> Self {
        Self {
            running: Mutex::new(true),
            class_time: Mutex::new(5400),
            rest_time: Mutex::new(1200),
            elapsed_time: Mutex::new(0),
            start_time: Mutex::new(Instant::now()),
            end_time: Mutex::new(Local::now()),
        }
    }

    fn sleep(& self, sleep_time: u64) {
        let mut past_time = 0;
        let sleep_step = Duration::from_millis(100);

        for _ in 0..(sleep_time * 10) {
            thread::sleep(sleep_step);
            // update elapsed_time from start_time
            let elapsed_secs = self.start_time.lock().unwrap().elapsed().as_secs();
            *self.elapsed_time.lock().unwrap() = elapsed_secs;
            past_time += 1;

            if !*self.running.lock().unwrap() {
                break;
            }

            if past_time % 20 == 0 {
                println!("past time: {}, elapsed_time: {}", past_time / 10, elapsed_secs);
            }
        }
    }

    fn rand_sleep(& self, start: u64, end: u64, text: &str) {
        let sleep_time = if end > start {
            rand::thread_rng().gen_range(start..=end)
        } else {
            0
        };
        println!("等待 {} 秒... {}", sleep_time, text);
        self.sleep(sleep_time);
    }

    fn play_sound(&self, file_path: &str) -> Result<()> {
        if !*self.running.lock().unwrap() {
            return Ok(());
        }

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let file = BufReader::new(File::open(file_path)?);
        let source = Decoder::new(file)?;
        let sink = Sink::try_new(&stream_handle)?;

        sink.append(source);
        sink.play();
        sink.sleep_until_end();

        Ok(())
    }

    fn sleep_play(& self, file: &str, start: u64, end: u64, name: &str) {
        self.rand_sleep(start, end, name);
        if let Err(e) = self.play_sound(file) {
            eprintln!("播放音频失败: {}, 错误: {}", file, e);
        }
        println!("播放提示音 {} {}", name, file);
    }

    pub fn init_tick(& self) {
        *self.start_time.lock().unwrap() = Instant::now();
        *self.elapsed_time.lock().unwrap() = 0;
    }

    pub fn resume_tick(& self) {
        *self.running.lock().unwrap() = true;
        self.tick_while();
    }

    pub fn start_tick(& self) {
        *self.running.lock().unwrap() = true;
        self.init_tick();
        self.tick_while();
    }

    fn tick_while(& self) {
        let class_time = *self.class_time.lock().unwrap();
        let elapsed = *self.elapsed_time.lock().unwrap();
        let end = Local::now() + ChronoDuration::seconds((class_time - elapsed) as i64);
        *self.end_time.lock().unwrap() = end;

        while *self.running.lock().unwrap() {
            println!("开始新一轮循环...");
            if let Err(e) = self.play_sound("alert.mp3") {
                eprintln!("播放开始音频失败: {}", e);
            }

            loop {
                let elapsed_now = *self.elapsed_time.lock().unwrap();
                let class_time_now = *self.class_time.lock().unwrap();
                if !(elapsed_now < class_time_now && *self.running.lock().unwrap()) {
                    break;
                }

                println!("elapsed_time: {}", elapsed_now);

                // 随机等待 3-5 分钟，播放提示音
                self.sleep_play("alert.mp3", 180, 300, "提示音");

                // 随机等待 10-15 秒，播放重新学习提示音
                self.sleep_play("tick_study.mp3", 10, 15, "重新学习提示音");
            }

            println!("播放休息提示音 rest.mp3");
            if let Err(e) = self.play_sound("rest.mp3") {
                eprintln!("播放休息音频失败: {}", e);
            }

            let rest_time = *self.rest_time.lock().unwrap();
            let rest_second = if rest_time % 60 == 0 {
                String::new()
            } else {
                format!("{}秒", rest_time % 60)
            };

            println!(
                "休息 {} 分钟{}后，开始下一轮循环...",
                rest_time / 60,
                rest_second
            );

            self.sleep(rest_time);

            if *self.running.lock().unwrap() && *self.elapsed_time.lock().unwrap() >= *self.class_time.lock().unwrap() {
                self.init_tick();
            }
        }
        println!("本次课程循环结束。");
    }

    pub fn stop(& self) {
        *self.running.lock().unwrap() = false;
    }

    pub fn set_elapsed(& self, elapsed_time: u64) {
        *self.elapsed_time.lock().unwrap() = elapsed_time;
        *self.start_time.lock().unwrap() = Instant::now() - Duration::from_secs(elapsed_time);
    }
}
