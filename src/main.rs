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
use axum_server::tls_rustls::RustlsConfig;
use kdl::KdlDocument;
use std::{
    env,
    fs::File,
    io::{self, Read},
    net::SocketAddr,
    path::PathBuf,
    time::Duration,
};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{info, Level};

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    #[cfg(target_os = "linux")]
    {
        if libsystemd::logging::connected_to_journal() {
            use tracing_subscriber::prelude::*;
            let journald = tracing_journald::layer().expect("journald should be available");
            tracing_subscriber::registry().with(journald).init();
        } else {
            tracing_subscriber::fmt::init();
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        tracing_subscriber::fmt::init();
    }

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

    let mut config_file = File::open(config_file_path)?;
    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents)?;

    let config_doc = config_contents
        .parse::<KdlDocument>()
        .expect("config file should be valid KDL");
    let config_doc = config_doc
        .get("config")
        .and_then(|config| config.children())
        .expect("KDL configuration should have config node");

    let port = config_doc
        .get_arg("port")
        .and_then(kdl::KdlValue::as_i64)
        .and_then(|port| u16::try_from(port).ok())
        .unwrap_or(8080);

    let tls_config = tls_config(config_doc).await;

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

    let handle = axum_server::Handle::new();
    let _shutdown_future = shutdown_signal(handle.clone());

    info!(?port, "Application starting on port: {}", port);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    if let Some(tls_config) = tls_config {
        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        axum_server::bind(addr)
            .handle(handle)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    Ok(())
}

async fn tls_config(config: &KdlDocument) -> Option<RustlsConfig> {
    let tls_children = config.get("tls")?.children()?;
    let cert_path = tls_children.get_arg("cert_path")?.as_string()?;
    let key_path = tls_children.get_arg("key_path")?.as_string()?;

    let cert_path = PathBuf::from(cert_path);
    let key_path = PathBuf::from(key_path);

    RustlsConfig::from_pem_file(cert_path, key_path).await.ok()
}

async fn shutdown_signal(handle: axum_server::Handle) {
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

    info!("signal received, starting graceful shutdown");

    handle.graceful_shutdown(Some(Duration::from_secs(10)));
}
