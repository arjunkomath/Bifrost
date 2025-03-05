use std::env;

use actix_web::{
    dev::Service,
    get,
    http::header::{CacheControl, CacheDirective},
    middleware::{self, Logger},
    web, App, HttpMessage, HttpResponse, HttpServer, Responder,
};
use anyhow::Result;
use dotenv::dotenv;
use env_logger::Env;
use serde_json::json;
use utils::jwt;

mod crypto;
mod routes;
mod utils;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(
        "
    USAGE

        POST /v1/token
        get user token for a namespace

        POST /v1/secret
        create a secret

        GET /v1/secret/{id}
        get the secret using the id

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

    if env::var("JWT_SECRET").is_err() {
        panic!("JWT_SECRET is required");
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
                web::scope("/v1")
                    .service(routes::auth::create_token)
                    .service(
                        web::scope("/secret")
                            .wrap_fn(|req, srv| {
                                let auth_header = req.headers().get("Authorization");
                                let token = auth_header
                                    .and_then(|h| h.to_str().ok())
                                    .and_then(|s| s.strip_prefix("Bearer "))
                                    .and_then(|t| jwt::validate_token(t).ok());

                                match token {
                                    Some(token_data) => {
                                        req.extensions_mut().insert(token_data);
                                        srv.call(req)
                                    }
                                    None => Box::pin(async move {
                                        let error_response = HttpResponse::Unauthorized()
                                            .content_type("application/json")
                                            .json(json!({
                                                "error": "Invalid token",
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
