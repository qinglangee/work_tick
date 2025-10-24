// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint_rust_template::ClassTicker;
use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    // 创建课程计时器实例
    let mut ticker = ClassTicker::new();
    
    // 在新线程中运行计时器
    std::thread::spawn(move || {
        ticker.start_tick();
    });

    // 创建并运行UI
    let ui = AppWindow::new()?;
    ui.run()?;

    Ok(())
}
