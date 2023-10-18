use std::sync::Arc;
use reqwest::{Error, header, Proxy};
use reqwest::blocking::{Client, ClientBuilder};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

static SPH_ID:&str = "5220";
static USE_DEV_PROXY:bool = false;

#[derive(Serialize, Deserialize, Default)]
pub struct SphAuthentication{
    pub sid:String,
    pub sph_session:String,
}

impl SphAuthentication {

    pub fn empty() -> Self{SphAuthentication::default()}

    fn build_cookies(&self) -> String {
        format!("SPH-Session={}; sid={}", self.sph_session, self.sid)
    }

    ///Creates a token that contains both sid and sph_session (format: <sid>-<sph_session>)
    pub fn as_token(&self) -> String {
        format!("{}-{}", self.sid, self.sph_session)
    }

    pub fn from_cookies(sid:String, sph_session:String) -> Self {
        SphAuthentication{
            sid,
            sph_session,
        }
    }

    ///Takes a token formatted as following (<sid>-<sph_session>) and creates a SphAuthentication from it
    pub fn from_token(token: &str) -> Self {
        let token_string = token.to_string();
        let segments: Vec<_> = token_string.split("-").collect();
        let (sid, sph_session) = (segments[0], segments[1]);
        SphAuthentication{
            sid:sid.to_string(),
            sph_session: sph_session.to_string(),
        }
    }
}

pub struct SphClient{
    client:Client,
    cookie_store:Arc<CookieStoreMutex>,
}

impl SphClient {
    pub fn new() -> Self {

        let cookie_store = CookieStoreMutex::new(CookieStore::default());
        let cookie_store = Arc::new(cookie_store);
        let mut client_builder = ClientBuilder::new()
            .cookie_store(true)
            .cookie_provider(std::sync::Arc::clone(&cookie_store));

        if USE_DEV_PROXY {
            client_builder = client_builder.danger_accept_invalid_certs(true)
            .proxy(Proxy::http("http://localhost:8080").unwrap())
            .proxy(Proxy::https("http://localhost:8080").unwrap());
        }

        SphClient{
            client: client_builder.build().unwrap(),
            cookie_store
        }
    }

    fn perform_request(&self, url:&str, cookies:&str, content:&str) -> Result<String, reqwest::Error>{
        let mut res = self.client.get(url)
            .body(content.to_string());
        if cookies != "" {
            res = res.header(header::COOKIE, cookies);
        }
        Ok(res.send()?.text()?)
    }

    fn _print_current_cookies(&self) {
        println!("Currently stored Cookies: ");
        let cookies = self.cookie_store.lock().unwrap();
        for cc in cookies.iter_any(){
            println!("{:?}", cc);
        }
    }

    /// Generates a session id ("sid") and SPH Session ID ("SPH-Session") by simulating a login using the given user credentials
    pub fn login(&self, username:&str, password:&str) -> Result<SphAuthentication, Error>{
        let url = format!("https://login.schulportal.hessen.de/?i={SPH_ID}");

        self.client.post(url.as_str()).form(&[
            ("user2", username),
            ("user", format!("{SPH_ID}.{username}").as_str()),
            ("password", password)
        ]).send()?;

        let cookies =self.cookie_store.lock().unwrap().clone();
        self.cookie_store.lock().unwrap().clear();

        let extract_auth_cookies = ||-> Result<(String, String), Error> {
            Ok((
                cookies.get("schulportal.hessen.de", "/", "sid").expect("sid cookie not supplied").value().to_string(),
                cookies.get("hessen.de", "/", "SPH-Session").expect("SPH-Session cookie not supplied").value().to_string()
            ))
        };
        let cookies = extract_auth_cookies()?;
        Ok(SphAuthentication::from_cookies(cookies.0, cookies.1))
    }


    ///Date Formatting: ddMMyyyy (Example: 22_09_2023)
    pub fn pulldown_subplan(&self, auth:&SphAuthentication, date:&str) -> Result<Value, Error> {
        let response = self.perform_request(
            "https://start.schulportal.hessen.de/vertretungsplan.php",
            auth.build_cookies().as_str(), "")?;

        let page = Html::parse_document(response.as_str());
        let selector_rows = Selector::parse(format!(r#"#vtable{} > tbody > tr"#, date).as_str()).unwrap();
        let rows = page.select(&selector_rows);

        let mut _len = 0;

        let mut result:Vec<[String; 8]> = Vec::new();

        let entry_selector = Selector::parse(r#"td"#).unwrap();

        for row in rows{
            _len += 1;
            let mut result_row:[String; 8] = Default::default();
            let entries = row.select(&entry_selector);
            for (index, entry) in entries.enumerate(){
                let text = entry.text().collect::<String>().trim().to_owned();
                result_row[index] = text;
            }
            result.push(result_row);
        }


        Ok(json!(result))
    }

    /// Check if a given SphAuthentication is still valid
    pub fn check_validity(&self, auth:&SphAuthentication) -> Result<bool, Error> {
        self.client.get("https://start.schulportal.hessen.de/").header("cookie", auth.build_cookies()).send()?;

        let cookies =self.cookie_store.lock().unwrap().clone();
        self.cookie_store.lock().unwrap().clear();

        let cookie_i = cookies.get("start.schulportal.hessen.de", "/", "i").expect("cookie i not found").value().to_string();

        Ok(cookie_i!="0")
    }
}
