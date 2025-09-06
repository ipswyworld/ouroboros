// bootstrap-server.rs
use actix_web::{get, App, HttpServer, HttpResponse, Responder};
use std::env;

#[get("/bootstrap.txt")]
async fn bootstrap() -> impl Responder {
    // simple: read a local file bootstrap.txt in the same dir, fallback to env BOOTSTRAP_LIST
    let contents = std::fs::read_to_string("bootstrap.txt")
        .or_else(|_| {
            Ok(env::var("BOOTSTRAP_LIST").unwrap_or_else(|_| "".into()))
        })
        .unwrap_or_default();
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(contents)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind = std::env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".into());
    println!("Serving bootstrap on http://{}/bootstrap.txt", bind);
    HttpServer::new(|| App::new().service(bootstrap))
        .bind(&bind)?
        .run()
        .await
}
