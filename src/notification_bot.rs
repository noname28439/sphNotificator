use std::alloc::handle_alloc_error;
use std::fmt::Error;
use chrono::{DateTime, Local, NaiveDate};
use log::info;
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
    cached_subplan: Vec<Value>,
    message_provider: Messenger,
    date:String,
    config: Configuration,
}

impl Bot {

    pub fn new(config:Configuration)->Self{
        Bot{
            database: Client::connect(&*config.database_credentials, NoTls).expect("Connection failed..."),
            sph_client: SphClient::new(),
            sph_session: SphAuthentication::empty(),
            cached_subplan: vec![],
            date: format_date(&Local::now()),
            message_provider: Messenger::new(&*config.messenger_endpoint, &*config.messenger_token),
            config
        }
    }

    fn handle_new_entry(&mut self, entry:&Vec<Value>) -> Result<(), Box<dyn std::error::Error>>{
        let class = entry.get(4).ok_or(Error)?.as_str().ok_or(Error)?;


        let affected_accounts = self.database.query("SELECT discord_userid FROM sph_notifications WHERE classes LIKE $1", &[&format!("%{}%", &class)])?;
        for affected in affected_accounts{
            let uid:i64 = affected.get(0);

            let _success = self.message_provider.send_message(uid, &*super::messenger::build_text(&entry).ok_or(Error)?).expect("message send failed");

        }
        Ok(())
    }

    fn handle_date_change(&mut self, new_date:&String){
        info!("Date changed from {} to {}", self.date, new_date);
        self.database.execute("INSERT INTO subplan_dumps VALUES ($1, $2)", &[
            &NaiveDate::parse_from_str(&self.date, DATE_FORMATTER).unwrap(),
            &json!(self.cached_subplan)]).expect("plan dump failed");
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let current_date = format_date(&Local::now());
        if current_date!=self.date {
            self.handle_date_change(&current_date);
            self.date = current_date;
        }

        //Check if session in active
        if !self.sph_client.check_validity(&self.sph_session)? {
            self.sph_session = self.sph_client.login(&*self.config.sph_cred_username, &*self.config.sph_cred_password)?;
            info!("New Session: {}", self.sph_session.as_token());
        }

        let sub_plan = self.sph_client.pulldown_subplan(&self.sph_session, &*format_date(&Local::now()))?;
        let sub_plan = sub_plan.as_array().ok_or(Error)?;

        for entry in sub_plan{
            if !self.cached_subplan.contains(entry) {
                self.cached_subplan.push(entry.clone());
                let entry = entry.as_array().ok_or(Error)?;

                self.handle_new_entry(entry).unwrap();

            }
        }
        self.handle_date_change(&"01_01_1999".to_string());
        Ok(())
    }

    pub fn _test(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let res = self.database.query("SELECT * FROM sph_notifications", &[])?;
        for row in res{
            let name:String = row.get(0);
            let discord_uid:i64 = row.get(1);
            let classes:String = row.get(2);


        }


        Ok(())
    }
}


pub fn format_date(date:&DateTime<Local>) -> String {
    date.format(DATE_FORMATTER).to_string()
}
