use crate::data::incident_repository::IncidentRepositoryData;
use actix_web::get;
use actix_web::{HttpResponse, Responder};
use chrono::{TimeZone, Utc};
use rss::{ChannelBuilder, GuidBuilder, ItemBuilder};

async fn rss_feed(incidents: IncidentRepositoryData) -> impl Responder {
    let items = incidents
        .get_all_incidents()
        .iter()
        .map(|i| (i, incidents.get_status_updates_by_incident(i)))
        .map(|(incident, status)| {
            let url = format!(
                "{}/incident/{}",
                crate::settings::get_host_url(),
                incident.id
            );

            let guid = GuidBuilder::default()
                .value(url.clone())
                .permalink(true)
                .build();

            let description = status
                .iter()
                .map(|(status, type_)| format!("<p>{} - {}</p>", type_.title, status.message))
                .fold(String::new(), |a, b| format!("{}\n{}", a, b));

            ItemBuilder::default()
                .title(format!(
                    "Incident on {}",
                    incident.formatted_creation_time()
                ))
                .content(escaper::encode_minimal(&description))
                .link(url)
                .guid(guid)
                //todo:  Can 2822 be used as a drop in replacement for 822?
                .pub_date(Utc.from_utc_datetime(&incident.created).to_rfc2822())
                .build()
        })
        .collect::<Vec<_>>();

    let channel = ChannelBuilder::default()
        .title("YouUp incidents for 'test'")
        .link("test")
        .description("Incidents for 'test'")
        .items(items)
        .build();

    HttpResponse::Ok()
        .append_header((http::header::CONTENT_TYPE.as_str(), "application/rss+xml"))
        .body(channel.to_string())
}

#[get("/feed/rss")]
pub async fn get_rss_feed(pool: IncidentRepositoryData) -> impl Responder {
    rss_feed(pool).await
}
