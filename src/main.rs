// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint_rust_template::ClassTicker;
use std::{error::Error, sync::{Arc, Mutex}, str::FromStr, thread, time::Duration};
use slint::SharedString;

slint::include_modules!();


fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
