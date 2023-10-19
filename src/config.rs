use std::fmt::Error;

#[derive(Debug, Clone)]
pub struct Configuration{
    pub tick_interval:i64,
    pub sph_cred_username:String,
    pub sph_cred_password:String,
    pub database_credentials: String,
    pub messenger_token: String,
    pub messenger_endpoint: String,
}

impl Configuration {
    pub fn load()->Result<Self, Box<dyn std::error::Error>>{

        let unpack_string =|val:&serde_json::Value| -> String {
            String::from(val.as_str().ok_or("").unwrap())
        };

        let f = std::fs::File::open("config.yaml")?;
        let data: serde_json::Value = serde_yaml::from_reader(f)?;

        let val_database = data["database"].as_object().expect("invalid database configuration");
        let db_credentials = val_database.iter().flat_map(
            |(k, v)| {k.chars().chain(['=']).chain(v.as_str().expect("invalid str").chars()).chain([' '])}
        ).collect();


        Ok(Configuration{
            tick_interval: data["tick_interval"].as_i64().ok_or(Error)?,
            sph_cred_username: unpack_string(&data["sph_credentials"]["username"]),
            sph_cred_password: unpack_string(&data["sph_credentials"]["password"]),
            database_credentials: db_credentials,
            messenger_token: unpack_string(&data["messenger"]["access_token"]),
            messenger_endpoint: unpack_string(&data["messenger"]["endpoint"]),
        })
    }
}