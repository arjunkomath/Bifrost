use std::env;

use actix_web::{
    dev::Service,
    get,
    http::header::{CacheControl, CacheDirective},
    middleware::{self, Logger},
    web, App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use dotenv::dotenv;
use env_logger::Env;
use serde_json::json;

mod crypto;
mod routes;
mod utils;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(
        "
    USAGE

        POST /v1/secret/{key}
        create a secret

        GET /v1/secret/{key}
        get secret using the key

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
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap_or(8080);

    let encryption_key = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY is required");
    if encryption_key.len() != 32 {
        panic!("ENCRYPTION_KEY must be 32 bytes long");
    }

    if env::var("API_KEY").is_err() {
        panic!("API_KEY is required");
    }

    if env::var("SQLITE_PATH").is_err() {
        panic!("SQLITE_PATH is required");
    }

    println!("Starting Bifrost on port {}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T")
                    .exclude("/health"),
            )
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))))
            .service(hello)
            .service(health)
            .service(
                web::scope("/v1").service(
                    web::scope("/secret")
                        .wrap_fn(|req, srv| {
                            let api_key = req.headers().get("x-api-key").unwrap();

                            match api_key.to_str().unwrap() == env::var("API_KEY").unwrap() {
                                true => srv.call(req),
                                false => Box::pin(async move {
                                    let error_response = HttpResponse::Unauthorized()
                                        .content_type("application/json")
                                        .json(json!({
                                            "error": "Invalid API key",
                                        }));

                                    Ok(req.into_response(error_response))
                                }),
                            }
                        })
                        .service(routes::secret::create)
                        .service(routes::secret::get),
                ),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}
