use crate::db::Database;
use crate::diesel::RunQueryDsl;
use crate::models::Setting;
use diesel::{ExpressionMethods, QueryDsl};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::ops::Deref;

pub fn get_host_protocol() -> String {
    env::var("HOST_PROTOCOL").unwrap_or_else(|_| "http".to_string())
}

pub fn get_host_ip() -> String {
    env::var("HOST_IP").unwrap_or_else(|_| "0.0.0.0".to_string())
}

pub fn get_host_port() -> String {
    env::var("HOST_PORT").unwrap_or_else(|_| "8102".to_string())
}

pub fn get_host_domain() -> String {
    env::var("HOST_DOMAIN").unwrap_or_else(|_| format!("{}:{}", get_host_ip(), get_host_port()))
}

pub fn get_host_url() -> String {
    format!("{}://{}", get_host_protocol(), get_host_domain())
}

pub fn get_history_size() -> usize {
    env::var("HISTORY_SIZE")
        .unwrap_or_else(|_| "".to_string())
        .parse::<usize>()
        .unwrap_or(30)
}

/// The minimum number of minutes of downtime that must occur for it to be counted
pub fn get_minimum_downtime_minutes() -> i64 {
    2
}

pub fn get_email_addr() -> String {
    env::var("ALERT_EMAIL").unwrap_or_else(|_| "".to_string())
}

pub fn smtp_username() -> String {
    env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string())
}

pub fn smtp_password() -> String {
    env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".to_string())
}

pub fn admin_username() -> String {
    env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_string())
}

lazy_static! {
    static ref ADMIN_PASSWORD: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(15)
        .collect();
}

lazy_static! {
    static ref PRIVATE_KEY: [u8; 32] = thread_rng().gen::<[u8; 32]>();
}

pub fn admin_password() -> String {
    env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
        println!("No admin password supplied, using '{}', set the ADMIN_PASSWORD environment variable to change to a persistent value.", ADMIN_PASSWORD.deref());
        ADMIN_PASSWORD.clone()
    })
}

pub fn private_key() -> Vec<u8> {
    env::var("PRIVATE_KEY")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| PRIVATE_KEY.to_vec())
}

pub struct PersistedSettings {
    db: Database,
}

pub const CUSTOM_SCRIPT: &str = "CUSTOM_SCRIPT";
pub const CUSTOM_STYLE: &str = "CUSTOM_STYLE";
pub const CUSTOM_HTML: &str = "CUSTOM_HTML";

impl PersistedSettings {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn get_setting(&self, name: &str) -> String {
        use crate::schema::settings;
        let setting: Vec<Setting> = settings::table
            .filter(settings::dsl::name.eq(name))
            .load::<Setting>(&self.db.get().unwrap())
            .unwrap();
        setting
            .first()
            .map(|f| f.value.clone())
            .unwrap_or_else(|| "".to_string())
    }
}
