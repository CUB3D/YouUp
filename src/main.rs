use crate::models::{NewStatus, Project, Status};
use actix_files::Files;
use actix_rt::spawn;

use actix_web::middleware::{Compress, Logger};
use actix_web::web::{resource, Data};
use actix_web::HttpServer;
use actix_web::{App, HttpResponse, Responder};
use askama::Template;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{Connection, ExpressionMethods, MysqlConnection, RunQueryDsl};
use dotenv::dotenv;
use http::status::StatusCode;
use reqwest::Client;
use std::env;

use chrono::Utc;
use core::fmt;
use diesel::query_dsl::methods::OrderDsl;
use serde::export::Formatter;
use std::convert::TryInto;
use std::ops::Sub;
use std::time::{Duration, Instant};

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
embed_migrations!();

pub mod models;
pub mod schema;

type Database = Pool<ConnectionManager<MysqlConnection>>;

fn get_db_connection() -> Database {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())
        .expect("Unable to run migrations");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    diesel::r2d2::Pool::builder()
        .max_size(4)
        .test_on_check_out(true)
        .build(manager)
        .unwrap()
}

async fn run_update_job(db: Database) {
    let c = Client::builder().build().unwrap();

    loop {
        use self::schema::projects::dsl::*;
        use self::schema::status as stat;

        let projects_list = projects
            .load::<Project>(&db.get().unwrap())
            .expect("Unable to load projects");

        for domain in &projects_list {
            // Check if domain is up, store in db and wait

            let req_start_time = Instant::now();
            let req = c.head(&domain.url).send().await;
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

enum ProjectStatusTypes {
    Operational,
    Failing,
    Failed,
    Unknown,
}

impl fmt::Display for ProjectStatusTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ProjectStatusTypes::Operational => "Operational",
            ProjectStatusTypes::Failing => "Failing",
            ProjectStatusTypes::Failed => "Failed",
            ProjectStatusTypes::Unknown => "Unknown",
        })
    }
}

struct StatusDay<'a> {
    status: Vec<&'a Status>,
}

impl StatusDay<'_> {
    fn get_overall_status(&self) -> ProjectStatusTypes {
        if self.status.is_empty() {
            ProjectStatusTypes::Unknown
        } else if self.status.iter().all(|x| x.status_code == 200) {
            ProjectStatusTypes::Operational
        } else if self.status.iter().all(|x| x.status_code != 200) {
            ProjectStatusTypes::Failed
        } else {
            //TODO: apply more logic here, should really only be failed if more fail than success (based on amount of time)

            ProjectStatusTypes::Failing
        }
    }
}

struct ProjectStatus<'a> {
    project: Project,
    days: Vec<StatusDay<'a>>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    projects: Vec<ProjectStatus<'a>>,
}

pub async fn root(pool: Data<Database>) -> impl Responder {
    use self::schema::projects::dsl::*;
    use self::schema::status as stat;

    //TODO: make number of days variable
    let projects_list = projects
        .load::<Project>(&pool.get().unwrap())
        .expect("Unable to load projects");
    let status_list = stat::dsl::status
        .order(stat::dsl::time.desc())
        .load::<Status>(&pool.get().unwrap())
        .expect("Unable to load status");

    let mut p = Vec::new();
    for proj in projects_list {
        let day_count = 5;
        let mut days: Vec<StatusDay> = Vec::with_capacity(day_count);
        for x in (0..day_count).rev() {
            let now = Utc::now().date();
            let then = now
                .sub(chrono::Duration::days(x.try_into().unwrap()))
                .naive_utc();
            log::info!("Date: {} - {} = {}", now, x, then);

            let status_on_day = status_list
                .iter()
                .filter(|s| s.project == proj.id && s.created.date() == then)
                .collect();

            days.push(StatusDay {
                status: status_on_day,
            });
        }

        p.push(ProjectStatus {
            project: proj,
            days,
        })
    }

    let template = IndexTemplate { projects: p }
        .render()
        .expect("Unable to render template");

    HttpResponse::Ok().body(template)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let db = get_db_connection();

    spawn(run_update_job(db.clone()));

    HttpServer::new(move || {
        App::new()
            .data(db.clone())
            .service(Files::new("/static", "./static"))
            .service(resource("/").to(root))
            .wrap(Logger::default())
            .wrap(Compress::default())
    })
    .bind("0.0.0.0:8102")?
    .run()
    .await
}
