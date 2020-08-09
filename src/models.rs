use super::schema::incident_status_type;
use super::schema::incident_status_update;
use super::schema::incidents;
use super::schema::status;
use chrono::{SecondsFormat, TimeZone, Utc};
use http::StatusCode;

#[derive(Queryable, Clone)]
pub struct Project {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub description: Option<String>,
    pub created: chrono::NaiveDateTime,
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

#[derive(Insertable)]
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

#[derive(Identifiable, Queryable, Clone)]
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
