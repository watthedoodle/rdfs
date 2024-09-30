use crate::config;
use std::{thread, time::Duration};

pub fn init() {
    println!("{}", crate::LOGO);

    if let Some(config) = config::get() {
        println!("==> launching node in [worker] mode...");
        loop {
            thread::sleep(Duration::from_millis(4000));
        }
    } else {
        println!("==> Error: unable able to load the valid cluster configuration. Please make sure the ENV 'RDFS_ENDPOINT' and 'RDFS_TOKEN' are set")
    }
}
