#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint_rust_template::ClassTicker;
use std::{error::Error, sync::{Arc}, str::FromStr, thread, time::Duration};
use slint::SharedString;
use chrono::Local;

slint::include_modules!();

#[derive(Clone)]
struct TickerHandler {
    ui: slint::Weak<AppWindow>,
    ticker: Arc<ClassTicker>,
}

impl TickerHandler {
    fn new(ui: &AppWindow, ticker: ClassTicker) -> Arc<Self> {
        Arc::new(Self {
            ui: ui.as_weak(),
            ticker: Arc::new(ticker),
        })
    }

    fn show_message(&self, msg: impl Into<SharedString>) {
        if let Some(ui) = self.ui.upgrade() {
            ui.set_message(msg.into());
        }
    }

    // fn with_ui<F, T>(&self, f: F) -> Option<T>
    // where
    //     F: FnOnce(&AppWindow) -> T
    // {
    //     self.ui.upgrade().map(f)
    // }

    fn update_ui(&self) {
        let t = &self.ticker;

        // read inner values via locks
        let class_time_val = *t.class_time.lock().unwrap();
        let elapsed_val = *t.elapsed_time.lock().unwrap();
        let rest_time_val = *t.rest_time.lock().unwrap();


        // Update message with current status
        let end_time = Local::now() + chrono::Duration::seconds((class_time_val - elapsed_val) as i64);
        let next_start = end_time + chrono::Duration::seconds(rest_time_val as i64);

        let msg = format!(
            "Total: {} Elapsed: {}\n下课时间: {}\n下节上课: {}", 
            class_time_val, 
            elapsed_val,
            end_time.format("%Y-%m-%d %H:%M:%S"),
            next_start.format("%Y-%m-%d %H:%M:%S")
        );
        self.ui.upgrade_in_event_loop(move |ui|{
            
            ui.set_ticker_info(TickerInfo {
                class_time: class_time_val.to_string().into(),
                elapsed_time: elapsed_val.to_string().into(),
                rest_time: rest_time_val.to_string().into(),
            });
            ui.set_message(msg.into());
        }).unwrap();
    }

    fn handle_time_input<F>(&self, value: String, field: &str, f: F)
    where
        F: FnOnce(& ClassTicker, u64)
    {
        if let Ok(time) = u64::from_str(&value) {
            f(&self.ticker, time);
            self.show_message(format!("已设置{}为 {} 秒", field, time));
            self.update_ui();
        } else {
            self.show_message(format!("请输入有效的数字！"));
        }
    }

    fn start_ui_update(self: &Arc<Self>) {
        let handler = Arc::clone(self);
        thread::spawn(move || {
            loop {
                handler.update_ui();
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn setup_callbacks(self: &Arc<Self>, ui: &AppWindow) {
        let handle_tick = |handler: &Arc<Self>, action: fn(& ClassTicker), msg: &'static str| {
            handler.ticker.stop();
            let h = Arc::clone(handler);
            h.show_message(msg);
            move || {
                let ticker = Arc::clone(&(Arc::clone(&h).ticker));
                thread::spawn(move || {
                    action(&ticker);
                });
            }
        };

        let h = Arc::clone(self);
        ui.on_set_class_time(move |value| {
            h.handle_time_input(value.to_string(), "课时", |t, time| {
                *t.class_time.lock().unwrap() = time;
            });
        });

        let h = Arc::clone(self);
        ui.on_set_elapsed_time(move |value| {
            h.handle_time_input(value.to_string(), "已学时间", |t, time| t.set_elapsed(time));
        });

        let h = Arc::clone(self);
        ui.on_set_rest_time(move |value| {
            h.handle_time_input(value.to_string(), "休息时间", |t, time| {
                *t.rest_time.lock().unwrap() = time;
            });
        });

        // ui.on_start_tick(handle_tick(self, ClassTicker::start_tick, "开始计时..."));
        let h = Arc::clone(self);
        ui.on_start_tick(move ||{
            // self.ticker.lock().ok().map(|t|t.start_tick());
            let h = Arc::clone(&h);
            h.show_message("开始计时...");
            thread::spawn(move || {
                let t = &h.ticker;
                t.stop();
                t.start_tick();
            });
        });
        ui.on_resume_tick(handle_tick(self, ClassTicker::resume_tick, "继续计时..."));
        // let h = Arc::clone(self);
        // ui.on_resume_tick(move || {
        //     let h = Arc::clone(&h);
        //     h.show_message("继续计时...");
        //     thread::spawn(move || {
        //         let t = &h.ticker;
        //         t.stop();
        //         t.resume_tick();
        //     });
        // });

        let h = Arc::clone(self);
        ui.on_stop_tick(move || {
            h.ticker.stop();
            // h.with_ticker(|t| t.stop());
            h.show_message("计时已停止");
        });
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Create and initialize UI
    let ui = AppWindow::new()?;
    ui.set_ticker_info(TickerInfo {
        class_time: "5400".into(),
        elapsed_time: "0".into(),
        rest_time: "1200".into(),
    });

    // Create handler, setup callbacks and start UI update
    let handler = TickerHandler::new(&ui, ClassTicker::new());
    handler.setup_callbacks(&ui);
    handler.start_ui_update();

    // Run UI
    ui.run()?;

    // Cleanup
    handler.ticker.stop();

    Ok(())
}
