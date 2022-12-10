// Copyright 2022 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use axum::{
    http::header::HeaderName,
    routing::{any, get},
    Router,
};
use std::env;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{info, Level};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));

    tracing_subscriber::fmt::init();

    let request_id_header = HeaderName::from_static(gargantua::request_id::REQUEST_ID_HEADER_NAME);

    let app = Router::new()
        .route("/version", get(gargantua::version::get_version))
        .route("/health", get(gargantua::health::get_health))
        .fallback_service(
            any(gargantua::fallback::any_method_not_allowed)
                .get(gargantua::fallback::get_not_found),
        )
        .layer(
            ServiceBuilder::new()
                .set_request_id(
                    request_id_header.clone(),
                    gargantua::request_id::MakeRequestUuid,
                )
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        )
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        ),
                )
                .propagate_request_id(request_id_header),
        );

    info!(?port, "Application starting on port: {}", port);

    axum::Server::bind(&format!("0.0.0.0:{port}").parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
