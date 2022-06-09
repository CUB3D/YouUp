use super::schema::email_subscriptions;
use super::schema::incident_status_type;
use super::schema::incident_status_update;
use super::schema::incidents;
use super::schema::settings;
use super::schema::sms_subscriptions;
use super::schema::status;
use super::schema::webhook_subscriptions;
use chrono::{SecondsFormat, TimeZone, Utc};
use http::StatusCode;

#[derive(Queryable, Clone)]
pub struct Project {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub description: Option<String>,
    pub created: chrono::NaiveDateTime,
    pub enabled: bool,
}

impl Project {
    pub fn formatted_description(&self) -> String {
        self.description.clone().unwrap_or_else(|| "".to_string())
    }
}

#[derive(Queryable, Clone)]
pub struct Status {
    pub id: i32,
    pub project: i32,
    pub time: i32,
    pub status_code: i32,
    pub created: chrono::NaiveDateTime,
}

impl Status {
    pub(crate) fn is_success(&self) -> bool {
        StatusCode::from_u16(self.status_code as u16)
            .map(|s| s.is_success())
            .unwrap_or(false)
    }

    pub(crate) fn formatted_creation_time(&self) -> String {
        Utc.from_utc_datetime(&self.created)
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "status"]
pub struct NewStatus {
    pub project: i32,
    pub time: i32,
    pub status_code: i32,
}

#[derive(Queryable, Identifiable, Clone)]
#[table_name = "incidents"]
pub struct Incidents {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
    pub project: i32,
}

impl Incidents {
    pub(crate) fn formatted_creation_time(&self) -> String {
        Utc.from_utc_datetime(&self.created)
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

#[derive(Insertable)]
#[table_name = "incidents"]
pub struct NewIncident {
    pub project: i32,
}

#[derive(Identifiable, Queryable, Clone, Debug)]
#[table_name = "incident_status_type"]
pub struct IncidentStatusType {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
    pub colour: String,
    pub title: String,
}

#[derive(Identifiable, Queryable, Clone, Associations)]
#[belongs_to(Incidents, foreign_key = "incident")]
#[table_name = "incident_status_update"]
pub struct IncidentStatusUpdate {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
    pub status_type: i32,
    pub message: String,
    pub incident: i32,
}

impl IncidentStatusUpdate {
    pub(crate) fn formatted_creation_time(&self) -> String {
        Utc.from_utc_datetime(&self.created)
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

#[derive(Insertable)]
#[table_name = "incident_status_update"]
pub struct NewIncidentStatusUpdate {
    pub status_type: i32,
    pub message: String,
    pub incident: i32,
}

#[derive(Identifiable, Queryable, Clone)]
pub struct Setting {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
    pub name: String,
    pub value: String,
}

#[derive(Identifiable, Queryable, Clone)]
pub struct EmailSubscription {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
    pub email: String,
    pub confirmed: bool,
}

#[derive(Insertable)]
#[table_name = "email_subscriptions"]
pub struct NewEmailSubscription {
    pub email: String,
}

#[derive(Identifiable, Queryable, Clone)]
pub struct SmsSubscription {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
    pub phone_number: String,
    pub confirmed: bool,
}

#[derive(Identifiable, Queryable, Clone)]
pub struct WebhookSubscription {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
    pub url: String,
    pub enabled: bool,
}
