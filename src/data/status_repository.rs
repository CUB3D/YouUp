use crate::db::Database;
use crate::diesel::RunQueryDsl;
use crate::models::Status;
use actix_web::web::Data;
use diesel::dsl::sql;
use diesel::sql_types::Bool;
use diesel::{ExpressionMethods, QueryDsl};

pub type StatusRepositoryData = Data<Box<dyn StatusRepository>>;

pub trait StatusRepository {
    fn get_status_last_30_days(&self) -> Vec<Status>;

    fn get_status_last_90_days(&self) -> Vec<Status>;
}

impl StatusRepository for Database {
    fn get_status_last_30_days(&self) -> Vec<Status> {
        let status_list: Vec<_> = crate::schema::status::dsl::status
            .filter(sql::<Bool>("created > DATE_SUB(NOW(), INTERVAL 30 day)"))
            .order(crate::schema::status::dsl::created.desc())
            .load::<Status>(&mut self.get().unwrap())
            .expect("Unable to load status");

        status_list
    }

    fn get_status_last_90_days(&self) -> Vec<Status> {
        let status_list: Vec<_> = crate::schema::status::dsl::status
            .filter(sql::<Bool>("created > DATE_SUB(NOW(), INTERVAL 90 day)"))
            .order(crate::schema::status::dsl::created.desc())
            .load::<Status>(&mut self.get().unwrap())
            .expect("Unable to load status");

        status_list
    }
}
