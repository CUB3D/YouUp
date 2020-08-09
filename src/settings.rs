use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::ops::Deref;

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
    static ref rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
}

pub fn admin_password() -> String {
    env::var("ADMIN_PASSWORD").unwrap_or_else(|_| {
        println!("No admin password supplied, using '{}', set the ADMIN_PASSWORD environment variable to change to a persistent value.", rand_string.deref());
        rand_string.clone()
    })
}
