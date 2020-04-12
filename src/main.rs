#[macro_use]
extern crate actix_web;

use std::{env, io};

use actix_files as fs;
use actix_web::http::{header, StatusCode};
use actix_web::{guard, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};

use env_logger;
use serde::Deserialize;

#[get("/healthz")]
async fn healthz() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json("Okay"))
}

async fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/index.html")?.set_status_code(StatusCode::NOT_FOUND))
}

#[derive(Deserialize, Debug)]
struct Query {
    cmd: String,
}

fn reddit(query: &str) -> String {
    if query == "rd" {
        "https://reddit.com".to_string()
    } else {
        format!("https://reddit.com/r/{}", &query[3..])
    }
}

fn google(query: &str) -> String {
    match query {
        "gg" => "https://google.com".to_string(),
        _ => format!("https://google.com/search?q={}", &query[..]),
    }
}
async fn search(_req: HttpRequest, info: web::Query<Query>) -> HttpResponse {
    let mut cmd = info.cmd.as_str();
    if cmd.contains(' ') {
        let space_index = cmd.find(' ').unwrap_or_else(|| 0);
        cmd = &cmd[..space_index];
    }

    let redirect_url = match cmd {
        "tw" => String::from("https://twitter.com"),
        "rd" => reddit(&info.cmd.as_str()),
        _ => google(&info.cmd.as_str()),
    };
    HttpResponse::Ok()
        .status(StatusCode::SEE_OTHER)
        .set_header(header::LOCATION, redirect_url)
        .finish()
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug, actix_server=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(healthz)
            .service(web::resource("/search").route(web::get().to(search)))
            .default_service(
                web::resource("").route(web::get().to(p404)).route(
                    web::route()
                        .guard(guard::Not(guard::Get()))
                        .to(HttpResponse::MethodNotAllowed),
                ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
