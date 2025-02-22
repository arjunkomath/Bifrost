use std::env;
use actix_web::{
    get,
    http::header::{CacheControl, CacheDirective},
    middleware, web, App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use dotenv::dotenv;

mod crypto;
mod routes;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(
        "
    USAGE

        PUT /v1/token
        tokenize a string
        
        GET /v1/token/{key}
        get the tokenized string using the token key

        GET /health
            health check
    ",
    )
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .body("success")
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap_or(8080);

    let encryption_key = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY is required");
    if encryption_key.len() != 32 {
        panic!("ENCRYPTION_KEY must be 32 bytes long");
    }

    println!("Starting image server on port {}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))))
            .service(hello)
            .service(health)
            .service(
                web::scope("/v1")
                    .service(routes::token::create)
                    .service(routes::token::get),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
