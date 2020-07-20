#[macro_use]
extern crate actix_web;

use std::{env, io};

use actix_web::{
    web, App, HttpResponse, HttpServer, HttpRequest, Result
};
use actix_web::middleware::Logger;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use svg::Document;
use svg::node::Text as TextContent;
use svg::node::element::{Rectangle, Text, Group};

struct RequestInfo {
    pub id: i32,
    pub ip_address: String,
    pub user_agent: String,
}

impl RequestInfo {
    pub async fn create(request: RequestInfo, pool: &PgPool) -> Result<RequestInfo, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let request = sqlx::query("INSERT INTO requests (ip_address, user_agent) VALUES ($1, $2) RETURNING id, ip_address, user_agent")
            .bind(&request.ip_address)
            .bind(&request.user_agent)
            .map(|row: PgRow| {
                RequestInfo {
                    id: row.get(0),
                    ip_address: row.get(1),
                    user_agent: row.get(2)
                }
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(request)
    }
}

fn profile_badge(count: i32) -> Document {
    let leftRect = Rectangle::new()
        .set("width", 50)
        .set("height", 20)
        .set("fill", "#555");

    let rightRect = Rectangle::new()
        .set("width", 70)
        .set("height", 50)
        .set("x", 50)
        .set("fill", "#4c1");


    let title_text = Text::new()
        .set("x", 25)
        .set("y", 14)
        .add(TextContent::new("views"));

    let count_text = Text::new()
        .set("x", 85)
        .set("y", 14)
        .add(TextContent::new(format!("{}", count)));

    let text_container = Group::new()
        .set("fill", "#fff")
        .set("text-anchor", "middle")
        .set("font-family", "DejaVu Sans,Verdana,Geneva,sans-serif")
        .set("font-size", 11)
        .add(title_text)
        .add(count_text);

    return Document::new()
        .set("height", 20)
        .set("width", 120)
        .add(leftRect)
        .add(rightRect)
        .add(text_container);
}

async fn profile(req: HttpRequest, db_pool: web::Data<PgPool>) -> HttpResponse {
    let user_agent = req.headers().get("User-Agent")
                                        .map_or("-", |header_value| header_value.to_str().unwrap());
    let request = RequestInfo {
        id: 0,
        ip_address: req.connection_info().remote().unwrap().to_string(),
        user_agent: String::from(user_agent)
    };

    match RequestInfo::create(request, &db_pool).await {
        Ok(info) => {
            HttpResponse::Ok()
                .set_header("Cache-Control", "max-age=0, no-cache, no-store, must-revalidate")
                .content_type("image/svg+xml")
                .body(profile_badge(info.id).to_string())
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string())
    }
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("OK")
}

const DEFAULT_PORT: &str = "8080";
const DATABASE_URL: &str = "postgres://localhost:5432/profile_counter_dev";

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

    let database_url  = if let Ok(url) = env::var("DATABASE_URL") {
        url
    } else {
        DATABASE_URL.to_string()
    };
    println!("{}", database_url);
    let db_pool = PgPool::new(&database_url).await.expect("Unable to create database pool");
    
    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .wrap(Logger::default())
            .service(web::resource("/profile.svg").route(web::get().to(profile)))
            .service(web::resource("/").route(web::get().to(index)))
    })
    .bind(bind_addr)?
    .run()
    .await
}