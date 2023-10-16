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
use kdl::KdlDocument;
use std::{
    env,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{debug, info, Level};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    #[cfg(feature = "tracing-journald")]
    {
        use tracing_subscriber::prelude::*;
        let journald = tracing_journald::layer().expect("journald should be available");
        tracing_subscriber::registry().with(journald).init();
    }

    #[cfg(not(feature = "tracing-journald"))]
    {
        tracing_subscriber::fmt::init();
    }

    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let assets_dir = {
        let mut path = PathBuf::from(
            env::var("STATE_DIRECTORY").unwrap_or_else(|_| String::from("/var/lib/gargantua")),
        );
        path.push("static");
        path
    };
    let config_dir = PathBuf::from(
        env::var("CONFIGURATION_DIRECTORY").unwrap_or_else(|_| String::from("/etc/gargantua")),
    );
    let config_file_path = {
        let mut path = config_dir.clone();
        path.push("config.kdl");
        path
    };
    debug!(?config_file_path, "Config file path");

    let mut config_file = File::open(config_file_path)?;
    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents)?;

    let config_doc: KdlDocument = config_contents
        .parse()
        .expect("config file was not valid KDL");

    let config_desc = config_doc
        .get("config")
        .and_then(|config| config.children())
        .map(|config| config.get_args("description"))
        .and_then(|desc| desc.first().map(|v| v.as_string()))
        .flatten()
        .unwrap_or("No description");

    info!(config_desc, "Config description");

    let request_id_header = HeaderName::from_static(gargantua::request_id::REQUEST_ID_HEADER_NAME);

    let mut not_found = assets_dir.clone();
    not_found.push("not_found.html");

    let app = Router::new()
        .route("/version", get(gargantua::version::get))
        .route("/health", get(gargantua::health::get))
        .route("/env", get(gargantua::env::get))
        .nest_service(
            "/assets",
            ServeDir::new(assets_dir.clone()).not_found_service(ServeFile::new(not_found)),
        )
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

    Ok(())
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
        () = ctrl_c => {},
        () = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
