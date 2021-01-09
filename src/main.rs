// Copyright 2020 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::Serialize;
use std::env;
use warp::Filter;

#[derive(Debug, Serialize)]
struct VersionResponse {
    commit: String,
}

impl VersionResponse {
    fn new() -> Self {
        let commit = env::var("SOURCE_COMMIT").unwrap_or_else(|_| String::from(""));

        Self { commit }
    }
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    let filter = env::var("RUST_LOG")
        .unwrap_or_else(|_| String::from("tracing=info,warp=info,gargantua=info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let port = port.parse::<u16>().unwrap();

    let version = warp::path("version").and(warp::path::end()).map(|| {
        let version = VersionResponse::new();
        warp::reply::json(&version)
    });

    let health = warp::path("health").and(warp::path::end()).map(|| {
        let health = HealthResponse {
            status: String::from("ok"),
        };
        warp::reply::json(&health)
    });

    let not_found_cached_get = warp::get().map(|| {
        let reply = warp::reply();
        let reply = warp::reply::with_status(reply, warp::http::StatusCode::NOT_FOUND);
        let reply = warp::reply::with_header(
            reply,
            warp::http::header::CACHE_CONTROL,
            "public, max-age=21600, s-maxage=21600, must-revalidate",
        );
        let now = std::time::SystemTime::now();
        let reply = warp::reply::with_header(
            reply,
            warp::http::header::LAST_MODIFIED,
            httpdate::fmt_http_date(now),
        );
        let expires = now + std::time::Duration::from_secs(60 * 60);
        let reply = warp::reply::with_header(
            reply,
            warp::http::header::EXPIRES,
            httpdate::fmt_http_date(expires),
        );
        reply
    });

    let routes = warp::get()
        .and(version.or(health).map(|reply| {
            warp::reply::with_header(
                reply,
                hyper::header::CACHE_CONTROL,
                "no-store, max-age=0, s-maxage=0",
            )
        }))
        .or(not_found_cached_get)
        .with(warp::trace(|info| {
            use tracing::field::{display, Empty};
            use warp::http::header;

            // TODO: Return status code
            // TODO: Elapsed time
            // TODO: Error message (if any)
            // TODO: Request-ID

            let span = tracing::info_span!(
                "request",
                host = Empty,
                method = %info.method(),
                path = %info.path(),
                origin = Empty,
            );

            let headers = info.request_headers();

            if let Some(host) = headers.get(header::HOST).and_then(|v| v.to_str().ok()) {
                span.record("host", &display(host));
            }

            if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
                span.record("origin", &display(origin));
            }

            span
        }));

    log::info!("Application starting on port: {}", port);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
