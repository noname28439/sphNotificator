use std::fmt::Debug;
use std::ops::{Add, Sub};
use std::time::Duration;
use log::{info, Level};
use crate::config::Configuration;
use crate::notification_bot::Bot;

mod sph_scraper;
mod notification_bot;
mod messenger;
mod config;


fn main() {

    simple_logger::init_with_level(Level::Info).unwrap();

    let config = Configuration::from_env().unwrap();
    let tick_delay = config.tick_interval;
    info!("Starting with config: {:?}", config);

    let mut b = Bot::new(config);

    loop {
        let _ = b.tick().expect("tick failed :(");
        std::thread::sleep(Duration::from_secs((tick_delay * 60) as u64));
    }

}
