use std::time::Duration;
use crate::notification_bot::Bot;

mod sph_scraper;
mod notification_bot;

fn read_config() -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("config.yaml")?;
    let data: serde_json::Value = serde_yaml::from_reader(f)?;
    let val_username = String::from(data["username"].as_str().expect("Invalid username in config"));
    let val_password = String::from(data["password"].as_str().expect("Invalid password in config"));
    let val_database = data["database"].as_object().expect("Invalid database credentials");

    let db_credentials = val_database.iter().flat_map(
        |(k, v)| {k.chars().chain(['=']).chain(v.as_str().expect("invalid str").chars()).chain([' '])}
    ).collect();

    Ok((val_username, val_password, db_credentials))
}

fn main() -> Result<(), Box<dyn std::error::Error>>{

    let (username, password, db_credentials) = read_config()?;
    let mut b = Bot::new(username, password, db_credentials);

    loop {
        let _ = b.tick().expect("tick failed :(");
        std::thread::sleep(Duration::from_secs(5*60));
    }

}
