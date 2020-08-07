use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::{web::Form, HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;

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

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[post("/admin")]
pub async fn post_admin_login(id: Identity, form: Form<LoginRequest>) -> impl Responder {
    if form.username == "admin" && form.password == "password" {
        id.remember("admin".to_owned());

        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin/dashboard")
            .finish();
    }

    return HttpResponse::PermanentRedirect()
        .header(http::header::LOCATION, "/admin")
        .finish();
}
