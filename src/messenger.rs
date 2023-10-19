use std::fmt::Error;
use serde_json::{json, Value};

pub struct Messenger{
    access_token:String,
    endpoint: String
}

impl Messenger {
    pub fn new(endpoint:&str, access_token:&str) -> Self{
        Messenger{access_token: String::from(access_token), endpoint: String::from(endpoint)}
    }

    pub fn send_message(&self, uid:i64, text:&str) -> Result<bool, Box<dyn std::error::Error>>{
        let payload = json!({
            "authtoken": self.access_token.as_str(),
            "userID": uid,
            "message": text
        });


        let client = reqwest::blocking::Client::builder().build()?;
        let response_text = client.get(&self.endpoint).body(serde_json::to_string(&payload)?).send()?.text()?;
        let response_json:Value = serde_json::from_str(&response_text)?;

        let success = response_json["success"].as_bool().ok_or(Error)?;

        Ok(success)
    }
}


pub fn build_text(entry:&Vec<Value>) -> Option<String>{
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