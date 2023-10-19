use std::fmt::Error;
use chrono::{DateTime, Local};
use chrono::format::{DelayedFormat, StrftimeItems};
use log::info;
use postgres::{Client, NoTls};
use serde_json::{json, Value};
use crate::config::Configuration;
use crate::messenger::Messenger;
use crate::sph_scraper::{SphAuthentication, SphClient};

pub struct Bot{
    database: Client,
    sph_client: SphClient,
    sph_session: SphAuthentication,
    cached_subplan: Vec<Value>,
    message_provider: Messenger,
    config: Configuration,
}

impl Bot {

    pub fn new(config:Configuration)->Self{
        Bot{
            database: Client::connect(&*config.database_credentials, NoTls).expect("Connection failed..."),
            sph_client: SphClient::new(),
            sph_session: SphAuthentication::from_token("12er6344rfe6l92iomhurtaha6-30565e48893def14141426285be6922b02d1590233f96e4ec37c7b5c565bb4ce"),
            cached_subplan: vec![],
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

    fn format_date(&self, date:&DateTime<Local>) -> String {
        date.format("%d_%m_%Y").to_string()
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        //Check if session in active
        if !self.sph_client.check_validity(&self.sph_session)? {
            self.sph_session = self.sph_client.login(&*self.config.sph_cred_username, &*self.config.sph_cred_password)?;
            info!("New Session: {}", self.sph_session.as_token());
        }

        let sub_plan = self.sph_client.pulldown_subplan(&self.sph_session, &*self.format_date(&chrono::Local::now()))?;  //TODO: use current date
        let sub_plan = sub_plan.as_array().ok_or(Error)?;

        for entry in sub_plan{
            if !self.cached_subplan.contains(entry) {
                self.cached_subplan.push(entry.clone());
                let entry = entry.as_array().ok_or(Error)?;

                self.handle_new_entry(entry).unwrap();

            }
        }


        Ok(())
    }

    pub fn _test(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let res = self.database.query("SELECT * FROM sph_notifications;", &[])?;
        for row in res{
            let name:String = row.get(0);
            let discord_uid:i64 = row.get(1);
            let classes:String = row.get(2);


        }


        Ok(())
    }
}

