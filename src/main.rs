#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint_rust_template::ClassTicker;
use std::{error::Error, sync::{Arc, Mutex}, str::FromStr, thread, time::Duration};
use slint::SharedString;
use chrono::Local;

slint::include_modules!();

#[derive(Clone)]
struct TickerHandler {
    ui: slint::Weak<AppWindow>,
    ticker: Arc<Mutex<ClassTicker>>,
}

impl TickerHandler {
    fn new(ui: &AppWindow, ticker: ClassTicker) -> Arc<Self> {
        Arc::new(Self {
            ui: ui.as_weak(),
            ticker: Arc::new(Mutex::new(ticker)),
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

    fn with_ticker<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut ClassTicker) -> T
    {
        self.ticker.lock().ok().map(|mut t| f(&mut t))
    }

    fn update_ui(&self) {
        if let (Some(ui), Ok(t)) = (self.ui.upgrade(), self.ticker.lock()) {
            ui.set_ticker_info(TickerInfo {
                class_time: t.class_time.to_string().into(),
                elapsed_time: t.elapsed_time.to_string().into(),
                rest_time: t.rest_time.to_string().into(),
            });
            
            // Update message with current status
            let end_time = Local::now() + chrono::Duration::seconds(
                (t.class_time - t.elapsed_time) as i64
            );
            let next_start = end_time + chrono::Duration::seconds(t.rest_time as i64);

            let msg = format!(
                "Total: {} Elapsed: {}\n下课时间: {}\n下节上课: {}", 
                t.class_time, 
                t.elapsed_time,
                end_time.format("%Y-%m-%d %H:%M:%S"),
                next_start.format("%Y-%m-%d %H:%M:%S")
            );
            ui.set_message(msg.into());
        }
    }

    fn handle_time_input<F>(&self, value: String, field: &str, f: F)
    where
        F: FnOnce(&mut ClassTicker, u64)
    {
        if let Ok(time) = u64::from_str(&value) {
            self.with_ticker(|t| f(t, time));
            self.show_message(format!("已设置{}为 {} 秒", field, time));
            self.update_ui();
        } else {
            self.show_message(format!("请输入有效的数字！"));
        }
    }

    fn spawn_ticker_action<F>(&self, f: F)
    where
        F: FnOnce(&mut ClassTicker) + Send + 'static
    {
        let ticker = Arc::clone(&self.ticker);
        thread::spawn(move || {
            if let Ok(mut t) = ticker.lock() {
                f(&mut t);
            }
        });
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
        let handle_tick = |handler: &Arc<Self>, action: fn(&mut ClassTicker), msg: &'static str| {
            let h = Arc::clone(handler);
            move || {
                h.with_ticker(|t| t.stop());
                h.spawn_ticker_action(action);
                h.show_message(msg);
            }
        };

        let h = Arc::clone(self);
        ui.on_set_class_time(move |value| {
            h.handle_time_input(value.to_string(), "课时", |t, time| t.class_time = time);
        });

        let h = Arc::clone(self);
        ui.on_set_elapsed_time(move |value| {
            h.handle_time_input(value.to_string(), "已学时间", |t, time| t.set_elapsed(time));
        });

        let h = Arc::clone(self);
        ui.on_set_rest_time(move |value| {
            h.handle_time_input(value.to_string(), "休息时间", |t, time| t.rest_time = time);
        });

        ui.on_start_tick(handle_tick(self, ClassTicker::start_tick, "开始计时..."));
        ui.on_resume_tick(handle_tick(self, ClassTicker::resume_tick, "继续计时..."));

        let h = Arc::clone(self);
        ui.on_stop_tick(move || {
            h.with_ticker(|t| t.stop());
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
    handler.with_ticker(|t| t.stop());

    Ok(())
}
