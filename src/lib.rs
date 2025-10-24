use std::time::{Duration, Instant};
use std::thread;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use rand::Rng;
use chrono::{DateTime, Local, Duration as ChronoDuration};
use anyhow::Result;

pub struct ClassTicker {
    running: bool,
    class_time: u64,    // 默认 90 分钟（5400秒）
    rest_time: u64,     // 默认休息 20 分钟（1200秒）
    elapsed_time: u64,
    start_time: Instant,
    end_time: DateTime<Local>,
}

impl ClassTicker {
    pub fn new() -> Self {
        Self {
            running: true,
            class_time: 5400,
            rest_time: 1200,
            elapsed_time: 0,
            start_time: Instant::now(),
            end_time: Local::now(),
        }
    }

    fn sleep(&mut self, sleep_time: u64) {
        let mut past_time = 0;
        let sleep_step = Duration::from_millis(100);
        
        for _ in 0..(sleep_time * 10) {
            thread::sleep(sleep_step);
            self.elapsed_time = self.start_time.elapsed().as_secs();
            past_time += 1;
            
            if !self.running {
                break;
            }
            
            if past_time % 20 == 0 {
                println!("past time: {}, elapsed_time: {}", past_time / 10, self.elapsed_time);
            }
        }
    }

    fn rand_sleep(&mut self, start: u64, end: u64, text: &str) {
        let sleep_time = if end > start {
            rand::thread_rng().gen_range(start..=end)
        } else {
            0
        };
        println!("等待 {} 秒... {}", sleep_time, text);
        self.sleep(sleep_time);
    }

    fn play_sound(&self, file_path: &str) -> Result<()> {
        if !self.running {
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

    fn sleep_play(&mut self, file: &str, start: u64, end: u64, name: &str) {
        self.rand_sleep(start, end, name);
        if let Err(e) = self.play_sound(file) {
            eprintln!("播放音频失败: {}, 错误: {}", file, e);
        }
        println!("播放提示音 {} {}", name, file);
    }

    pub fn init_tick(&mut self) {
        self.start_time = Instant::now();
        self.elapsed_time = 0;
    }

    pub fn resume_tick(&mut self) {
        self.running = true;
        self.tick_while();
    }

    pub fn start_tick(&mut self) {
        self.running = true;
        self.init_tick();
        self.tick_while();
    }

    fn tick_while(&mut self) {
        self.end_time = Local::now() + ChronoDuration::seconds(
            (self.class_time - self.elapsed_time) as i64
        );

        while self.running {
            println!("开始新一轮循环...");
            // if let Err(e) = self.play_sound("class_work.mp3") {
            if let Err(e) = self.play_sound("alert.mp3") {
                eprintln!("播放开始音频失败: {}", e);
            }

            while self.elapsed_time < self.class_time && self.running {
                println!("elapsed_time: {}", self.elapsed_time);
                
                // 随机等待 3-5 分钟，播放提示音
                self.sleep_play("alert.mp3", 180, 300, "提示音");

                // 随机等待 10-15 秒，播放重新学习提示音
                self.sleep_play("tick_study.mp3", 10, 15, "重新学习提示音");
            }

            println!("播放休息提示音 rest.mp3");
            if let Err(e) = self.play_sound("rest.mp3") {
                eprintln!("播放休息音频失败: {}", e);
            }

            let rest_second = if self.rest_time % 60 == 0 {
                String::new()
            } else {
                format!("{}秒", self.rest_time % 60)
            };
            
            println!(
                "休息 {} 分钟{}后，开始下一轮循环...",
                self.rest_time / 60,
                rest_second
            );
            
            self.sleep(self.rest_time);
            
            if self.running && self.elapsed_time >= self.class_time {
                self.init_tick();
            }
        }
        println!("本次课程循环结束。");
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn set_elapsed(&mut self, elapsed_time: u64) {
        self.elapsed_time = elapsed_time;
        self.start_time = Instant::now() - Duration::from_secs(elapsed_time);
    }
}
