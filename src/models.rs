use super::schema::status;
use http::StatusCode;

#[derive(Queryable)]
pub struct Project {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub description: Option<String>,
    pub created: chrono::NaiveDateTime,
}

#[derive(Queryable)]
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
}

#[derive(Insertable)]
#[table_name = "status"]
pub struct NewStatus {
    pub project: i32,
    pub time: i32,
    pub status_code: i32,
}
