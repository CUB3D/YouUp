use crate::models::{NewStatus, Project};

use diesel::RunQueryDsl;
use http::status::StatusCode;
use reqwest::Client;

use crate::db::Database;
use std::time::{Duration, Instant};

pub async fn run_update_job(db: Database) {
    let c = Client::builder().build().unwrap();

    loop {
        use crate::schema::projects::dsl::*;
        use crate::schema::status as stat;

        let projects_list = projects
            .load::<Project>(&db.get().unwrap())
            .expect("Unable to load projects");

        for domain in &projects_list {
            // Check if domain is up, store in db and wait

            let req_start_time = Instant::now();
            let req = c.get(&domain.url).send().await;
            let req_duration = req_start_time.elapsed();
            let status = req.map(|v| v.status()).unwrap_or(StatusCode::NOT_FOUND);

            diesel::insert_into(stat::table)
                .values(NewStatus {
                    project: domain.id,
                    //TODO: change the type of this field
                    time: req_duration.as_millis() as i32,
                    status_code: status.as_u16() as i32,
                })
                .execute(&db.get().unwrap())
                .unwrap();
        }

        tokio::time::delay_for(Duration::from_secs(90)).await;
    }
}