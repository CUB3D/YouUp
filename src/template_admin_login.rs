use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::{HttpResponse, Responder};
use askama::Template;

#[derive(Template)]
#[template(path = "admin_login.html")]
pub struct LoginTemplate {}

#[get("/admin")]
pub async fn get_admin_login(_id: Identity) -> impl Responder {
    let template = LoginTemplate {}
        .render()
        .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}

#[post("/admin")]
pub async fn post_admin_login(id: Identity) -> impl Responder {
    id.remember("admin".to_owned());

    let template = LoginTemplate {}
        .render()
        .expect("Unable to render template");
    HttpResponse::Ok().body(template)
}
