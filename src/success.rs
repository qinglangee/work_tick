use std::sync::LockResult;
use std::sync::MutexGuard;


// pub fn lock<T>(res: &LockResult<MutexGuard<'_, T>>, f: impl FnOnce()) 
// {
//     match res.lock() {
//         Ok(sender) => {
//             f();
//         },
//         Err(e) => {
//             eprintln!("Failed to acquire command_sender lock: {}", e);
//         }
//     }
// }
// pub fn lock_or<T>(&res : LockResult<MutexGuard<'_, T>>, f_ok, f_err) 
// {
//     match res.lock() {
//         Ok(sender) => {
//             sender.send(PlayerCommand::Stop).unwrap();
//         },
//         Err(e) => {
//             eprintln!("Failed to acquire command_sender lock: {}", e);
//         }
//     }
// }