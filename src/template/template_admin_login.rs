use crate::settings;
use crate::settings::{CUSTOM_SCRIPT, CUSTOM_STYLE, PersistedSettings};
use actix_identity::Identity;
use actix_web::post;
use actix_web::web::Data;
use actix_web::{HttpMessage, HttpRequest, get};
use actix_web::{HttpResponse, web::Form};
use askama::Template;
use serde::Deserialize;

pub trait AdminLogin {
    fn is_logged_in(&self) -> bool;
}

impl AdminLogin for Option<Identity> {
    fn is_logged_in(&self) -> bool {
        match self {
            Some(id) => matches!(id.id(), Ok(x) if x == "admin"),
            None => settings::insecure(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin_login.html")]
pub struct LoginTemplate {
    pub custom_script: String,
    pub custom_style: String,
}

#[get("/admin")]
pub async fn get_admin_login(
    id: Option<Identity>,
    settings: Data<PersistedSettings>,
) -> HttpResponse {
    settings::admin_password();

    if id.is_logged_in() {
        println!("Already logged in, sending to dashboard");
        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin/dashboard"))
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
pub async fn post_admin_login(request: HttpRequest, form: Form<LoginRequest>) -> HttpResponse {
    if form.username == settings::admin_username() && form.password == settings::admin_password() {
        Identity::login(&request.extensions(), "admin".into()).unwrap();

        return HttpResponse::PermanentRedirect()
            .append_header((http::header::LOCATION.as_str(), "/admin/dashboard"))
            .finish();
    }

    HttpResponse::PermanentRedirect()
        .append_header((http::header::LOCATION.as_str(), "/admin"))
        .finish()
}
