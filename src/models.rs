use super::schema::status;

#[derive(Queryable)]
pub struct Project {
    pub id: i32,
    pub url: String,
    pub name: String,
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

#[derive(Insertable)]
#[table_name = "status"]
pub struct NewStatus {
    pub project: i32,
    pub time: i32,
    pub status_code: i32,
}
