#[macro_use]
extern crate actix_web;

use std::{env, io};

use actix_web::{
    web, App, HttpResponse, HttpServer,
};
use actix_web::middleware::Logger;

async fn profile() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(format!("Hello!"))
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("OK")
}

const DEFAULT_PORT: &str = "8080";

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug, actix_server=info");
    let port = env::var("PORT").map_or(DEFAULT_PORT.to_string(), |port| {
        match port.is_empty() {
            true => DEFAULT_PORT.to_string(),
            false => port
        }
    });
    let bind_addr = format!("0.0.0.0:{}", port);
    env_logger::init();
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(web::resource("/profile.svg").route(web::get().to(profile)))
            .service(web::resource("/").route(web::get().to(index)))
    })
    .bind(bind_addr)?
    .run()
    .await
}