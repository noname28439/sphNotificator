use std::fmt::Debug;
use std::ops::{Add, Sub};
use std::time::Duration;
use chrono::format::{DelayedFormat, StrftimeItems};
use chrono::{DateTime, Local};
use log::{info, Level};
use simple_logger::SimpleLogger;
use crate::config::Configuration;
use crate::notification_bot::Bot;

mod sph_scraper;
mod notification_bot;
mod messenger;
mod config;


fn main() -> Result<(), Box<dyn std::error::Error>>{

    simple_logger::init_with_level(Level::Info).unwrap();

    let config = Configuration::load()?;
    info!("Starting with config: {:?}", config);

    let mut b = Bot::new(config);

    loop {
        let _ = b.tick().expect("tick failed :(");
        std::thread::sleep(Duration::from_secs(5*60));
    }

}
