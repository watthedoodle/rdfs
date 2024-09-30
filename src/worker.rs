use std::{thread, time::Duration};

pub fn init() {
    println!("{}", crate::LOGO);
    println!("==> launching node in [worker] mode...");

    loop {
        thread::sleep(Duration::from_millis(4000));
    }
}
