use crate::models::{NewStatus, Project, Status};

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use http::status::StatusCode;
use reqwest::Client;

use crate::db::Database;
use crate::settings;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
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

            let req = c.get(&domain.url).send();
            let req_start_time = Instant::now();
            let response = req.await;
            let req_duration = req_start_time.elapsed();
            let status = response
                .map(|v| v.status())
                .unwrap_or(StatusCode::NOT_FOUND);

            // Get the most recent status
            let most_recent_status = stat::table
                .filter(stat::dsl::project.eq(domain.id))
                .order_by(stat::dsl::created.desc())
                .limit(1)
                .load::<Status>(&db.get().unwrap());

            diesel::insert_into(stat::table)
                .values(NewStatus {
                    project: domain.id,
                    //TODO: change the type of this field
                    time: req_duration.as_millis() as i32,
                    status_code: status.as_u16() as i32,
                })
                .execute(&db.get().unwrap())
                .unwrap();

            if let Ok(stat) = most_recent_status {
                if let Some(stat2) = stat.first() {
                    if stat2.is_success() && !status.is_success() {
                        let email = Message::builder()
                            // Addresses can be specified by the tuple (email, alias)
                            .to(crate::settings::get_email_addr()
                                .parse()
                                .expect("Unable to parse alert email"))
                            .from("YouUp <alerts@you-up.net>".parse().unwrap())
                            .subject(format!("Alert in project '{}'", domain.name))
                            .body("Service is now down")
                            .unwrap();

                        let creds =
                            Credentials::new(settings::smtp_username(), settings::smtp_password());

                        // Open a remote connection to gmail
                        let mailer = SmtpTransport::relay("smtp.gmail.com")
                            .unwrap()
                            .credentials(creds)
                            .build();

                        // Send the email
                        let result = mailer.send(&email);

                        if result.is_ok() {
                            println!("Email sent");
                        } else {
                            println!("Could not send email: {:?}", result);
                        }
                    }
                }
            }
        }

        tokio::time::delay_for(Duration::from_secs(90)).await;
    }
}
