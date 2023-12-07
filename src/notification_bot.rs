use std::alloc::handle_alloc_error;
use std::fmt::Error;
use std::time::Duration;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, Utc};
use log::{info, warn};
use postgres::{Client, NoTls};
use serde_json::{json, Value};
use crate::config::Configuration;
use crate::messenger::Messenger;
use crate::sph_scraper::{SphAuthentication, SphClient};

static DATE_FORMATTER:&str = "%d_%m_%Y";

pub struct Bot{
    database: Client,
    sph_client: SphClient,
    sph_session: SphAuthentication,
    message_provider: Messenger,
    config: Configuration,
}

impl Bot {

    pub fn new(config:Configuration)->Self{
        Bot{
            database: Client::connect(&*config.database_credentials, NoTls).expect("Connection failed..."),
            sph_client: SphClient::new(),
            sph_session: SphAuthentication::empty(),
            message_provider: Messenger::new(&*config.messenger_endpoint, &*config.messenger_token),
            config
        }
    }

    fn handle_new_entry(&mut self, entry:&Vec<Value>) -> Result<(), Box<dyn std::error::Error>>{
        let class = entry.get(4).ok_or(Error)?.as_str().ok_or(Error)?;
        if class==""  {
            warn!("Empty class field...");
            return Ok(())
        }

        let affected_accounts = self.database.query("SELECT discord_userid, name FROM sph_notifications WHERE classes LIKE $1", &[&format!("%{}%", &class)])?;
        self.database.execute("insert into subplan_entries values ($1, $2)", &[&Local::now().naive_local(), &json!(entry)])?;
        for affected in affected_accounts{
            let uid:i64 = affected.get(0);
            let name:String = affected.get(1);
            let success = self.message_provider.send_message(uid, &*super::messenger::build_text(&entry).ok_or(Error)?)?;
            info!("notifying {}... success:{}", name, success);
        }
        Ok(())
    }

    fn check_subplan(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let sub_plan = self.sph_client.pulldown_subplan(&self.sph_session, &*subplan_format_date(&Local::now()))?;
        let sub_plan = sub_plan.as_array().ok_or(Error)?;

        let already_detected_entries = self.database.query("select * from subplan_entries where date_trunc('day', detection) = date_trunc('day', $1::timestamp)", &[&Local::now().naive_local()])?;
        let already_detected = |check:&Value|->bool{
            for cr in &already_detected_entries{
                let cmp:Value = cr.get(1);
                if &cmp == check {return true;}
            }
            return false;
        };

        for entry in sub_plan{
            if !already_detected(entry) {
                let entry = entry.as_array().ok_or(Error)?;
                info!("Detected new Entry {:?}", entry);

                self.handle_new_entry(entry).unwrap();
            }
        }

        Ok(())
    }

    fn check_sessions(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        if self.database.is_valid(Duration::from_secs(15)).is_err() {
            info!("Reconnecting to database...");
            self.database = Client::connect(&*self.config.database_credentials, NoTls).expect("Connection failed...")
        }

        //Check if session in active
        if !self.sph_client.check_validity(&self.sph_session)? {
            self.sph_session = self.sph_client.login(&*self.config.sph_cred_username, &*self.config.sph_cred_password)?;
            info!("New Session: {}", self.sph_session.as_token());
        }
        Ok(())
    }


    pub fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>>{

        self.check_sessions()?;
        self.check_subplan()?;

        Ok(())
    }

}


pub fn subplan_format_date(date:&DateTime<Local>) -> String {
    date.format(DATE_FORMATTER).to_string()
}
