use std::env;

use actix_web::{
    dev::Service,
    get,
    http::header::{CacheControl, CacheDirective},
    middleware, web, App, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use dotenv::dotenv;
use serde_json::json;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
        get secret

        DELETE /v1/secret/{key}
        delete secret

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

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap_or(8080);

    let encryption_key = env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY is required");
    if encryption_key.len() != 32 {
        panic!("ENCRYPTION_KEY must be 32 bytes long");
    }

    let required_env_vars = vec!["API_KEY", "TURSO_ORG", "TURSO_GROUP_TOKEN", "TURSO_REGION"];

    for var in required_env_vars {
        if env::var(var).is_err() {
            panic!("{} is required", var);
        }
    }

    info!("Starting Bifrost on port {}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(tracing_actix_web::TracingLogger::default())
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
                        .service(routes::secret::upsert)
                        .service(routes::secret::delete)
                        .service(routes::secret::get),
                ),
            )
    })
    .bind(("::", port))?
    .run()
    .await?;

    Ok(())
}
