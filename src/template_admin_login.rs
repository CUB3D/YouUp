use crate::settings::{PersistedSettings, CUSTOM_SCRIPT, CUSTOM_STYLE};
use actix_identity::Identity;
use actix_web::get;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{web::Form, HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;

pub trait AdminLogin {
    fn is_logged_in(&self) -> bool;
}

impl AdminLogin for Identity {
    fn is_logged_in(&self) -> bool {
        return self.identity() == Some("admin".to_string());
    }
}

#[derive(Template)]
#[template(path = "admin_login.html")]
pub struct LoginTemplate {
    pub custom_script: String,
    pub custom_style: String,
}

#[get("/admin")]
pub async fn get_admin_login(id: Identity, settings: Data<PersistedSettings>) -> impl Responder {
    crate::settings::admin_password();

    if id.is_logged_in() {
        println!("Already logged in, sending to dashboard");
        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin/dashboard")
            .finish();
    }

    let template = LoginTemplate {
        custom_script: settings.get_setting(CUSTOM_SCRIPT),
        custom_style: settings.get_setting(CUSTOM_STYLE),
    }
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
    if form.username == crate::settings::admin_username()
        && form.password == crate::settings::admin_password()
    {
        id.remember("admin".to_owned());

        return HttpResponse::PermanentRedirect()
            .header(http::header::LOCATION, "/admin/dashboard")
            .finish();
    }

    return HttpResponse::PermanentRedirect()
        .header(http::header::LOCATION, "/admin")
        .finish();
}
