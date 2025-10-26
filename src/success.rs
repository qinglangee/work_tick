//! 这个模块提供了一些辅助函数，用于简化对 Mutex 的锁定和消息发送操作。
//! 它们封装了常见的错误处理逻辑，避免在每次使用 Mutex 或发送消息时重复编写相同的代码。
//! 就是只管正确的执行路径， 忽略错误情况，并打印错误信息。
//! 
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::sync::MutexGuard;
use crate::audio_player::PlayerCommand;



/// 锁定给定的 Mutex，并在成功时执行提供的闭包。
/// 如果锁定失败，则打印错误信息并忽略该错误。
pub fn lock<T, F>(res: &Mutex<T>, f: F)
where 
    F: FnOnce(MutexGuard<'_, T>)
{
    match res.lock() {
        Ok(data) => {
            f(data);
        },
        Err(e) => {
            eprintln!("Ignored: failed to acquire lock: {}", e);
        }
    }
}


pub fn send(sender: &Sender<PlayerCommand>, cmd: PlayerCommand) {
    match sender.send(cmd) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Failed to send command: {}", e);
        }
    }
}

pub fn lock_send(res: &Mutex<Sender<PlayerCommand>>, cmd: PlayerCommand) {
    lock(res, |sender| {
        send(&sender, cmd);
    });
}