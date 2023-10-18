use std::fmt::Error;
use postgres::{Client, NoTls};
use serde_json::{json, Value};
use crate::sph_scraper::{SphAuthentication, SphClient};

fn send_message(uid:i64, text:&str) -> Result<bool, Box<dyn std::error::Error>>{

    let payload = json!({
        "authtoken": "xSqeogeoot1OkujAFRNH",
        "userID": uid,
        "message": text
    });

    let client = reqwest::blocking::Client::builder().build()?;

    let response_text = client.get("http://192.168.178.45:30080/api/").body(serde_json::to_string(&payload)?).send()?.text()?;
    let response_json:Value = serde_json::from_str(&response_text)?;

    let success = response_json["success"].as_bool().ok_or(Error)?;

    Ok(success)
}

fn build_text(entry:&Vec<Value>) -> Option<String>{
    return if entry[3].as_str()? == "Selbststudium" && entry[7].as_str()?.contains("Entfall") {
        Some(format!("**:no_entry:   {} entfällt! (Stunde {} bei {})**", entry[4].as_str()?, entry[0].as_str()?, entry[2].as_str()?))
    } else if entry[3].as_str()? == "Raum" {
        Some(format!("**:globe_with_meridians:   Raumwechsel in {} bei {} ({}. Stunde)**
    `{} -> {}`", entry[4].as_str()?, entry[2].as_str()?, entry[0].as_str()?, entry[6].as_str()?, entry[5].as_str()?))
    } else {
        Some(format!("**:warning:   Neuer, Dich betreffender, Vertretungsplan-Eintrag:**```fix
    Fach: {}
    Stunde: {}
    Lehrkraft: {}
    Art: {}
    Hinweis: {}```", entry[4], entry[0], entry[2], entry[3], entry[7]))
    }
}

pub struct Bot{
    database: Client,
    sph_client: SphClient,
    sph_session: SphAuthentication,
    sph_accountdata: (String, String),
    cached_subplan: Vec<Value>,
}

impl Bot {
    pub fn new(username:String, password:String, db_credentials:String)->Self{
        Bot{
            database: Client::connect(&*db_credentials, NoTls).expect("Connection failed..."),
            sph_client: SphClient::new(),
            sph_session: SphAuthentication::from_token("12er6344rfe6l92iomhurtaha6-30565e48893def14141426285be6922b02d1590233f96e4ec37c7b5c565bb4ce"),
            sph_accountdata: (username, password),
            cached_subplan: vec![],
        }
    }

    fn handle_new_entry(&mut self, entry:&Vec<Value>) -> Result<(), Box<dyn std::error::Error>>{
        let class = entry.get(4).ok_or(Error)?.as_str().ok_or(Error)?;


        let affected_accounts = self.database.query("SELECT discord_userid FROM sph_notifications WHERE classes LIKE $1", &[&format!("%{}%", &class)])?;
        for affected in affected_accounts{
            let uid:i64 = affected.get(0);

            let _success = send_message(uid, &*build_text(&entry).ok_or(Error)?).expect("message send crash");

        }
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        //Check if session in active
        if !self.sph_client.check_validity(&self.sph_session)? {
            self.sph_session = self.sph_client.login(&*self.sph_accountdata.0, &*self.sph_accountdata.1)?;
            println!("New Session: {}", self.sph_session.as_token());
        }

        let sub_plan = self.sph_client.pulldown_subplan(&self.sph_session, "18_10_2023")?;
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

