use crate::data::incident_repository::IncidentRepositoryData;
use actix_web::get;
use actix_web::{HttpResponse, Responder};
use atom_syndication::{ContentBuilder, EntryBuilder, FeedBuilder, LinkBuilder};
use chrono::{Offset, TimeZone, Utc};
use uuid::Uuid;

async fn atom_feed(incidents: IncidentRepositoryData) -> impl Responder {
    let entries = incidents
        .get_all_incidents()
        .iter()
        .map(|i| (i, incidents.get_status_updates_by_incident(i)))
        .map(|(incident, status)| {
            let url = format!(
                "{}/incident/{}",
                crate::settings::get_host_url(),
                incident.id
            );

            let description = status
                .iter()
                .map(|(status, type_)| format!("<p>{} - {}</p>", type_.title, status.message))
                .fold(String::new(), |a, b| format!("{}\n{}", a, b));

            EntryBuilder::default()
                .title(format!(
                    "Incident on {}",
                    incident.formatted_creation_time()
                ))
                .content(
                    ContentBuilder::default()
                        .value(description)
                        .content_type(Some("html".to_string()))
                        .build(),
                )
                .links(vec![LinkBuilder::default().href(url).build()])
                .id(Uuid::new_v4().to_string())
                .published(Utc.fix().from_local_datetime(&incident.created).unwrap())
                .build()
        })
        .collect::<Vec<_>>();

    let feed = FeedBuilder::default()
        .id(Uuid::new_v4().to_string())
        .title("YouUp incidents for 'test'")
        .subtitle(Some(atom_syndication::Text::plain("Incidents for 'test'")))
        .updated(Utc::now().with_timezone(&Utc))
        .links(vec![LinkBuilder::default().href("test").build()])
        .entries(entries)
        .build();

    HttpResponse::Ok()
        .append_header((http::header::CONTENT_TYPE, "application/atom+xml"))
        .body(feed.to_string())
}

#[get("/feed/atom")]
pub async fn get_atom_feed(pool: IncidentRepositoryData) -> impl Responder {
    atom_feed(pool).await
}
