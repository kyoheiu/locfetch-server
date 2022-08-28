mod router;

use axum::{
    routing::{get, post},
    Router,
};
use http::{HeaderValue, Method};
use router::*;
use tower_http::cors::{Any, CorsLayer, Origin};

#[tokio::main]
async fn main() {
    env_logger::init();

    let origins = vec![
        std::env::var("URL")
            .unwrap_or_else(|_| panic!("env var URL not found."))
            .parse::<HeaderValue>()
            .unwrap(),
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
    ];
    println!("Server started.");

    let app = Router::new()
        .route("/stats", post(stats))
        .route("/", get(health))
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::list(origins))
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any),
        );

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
