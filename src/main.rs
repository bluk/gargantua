// Copyright 2020 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::Serialize;
use std::env;
use std::str::FromStr;
use tide::{http::headers::HeaderName, Request, Response, StatusCode};

use gargantua::{log_request::LogRequest, request_id::RequestIdMiddleware};

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

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    env_logger::init().ok();

    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));

    let mut app = tide::new();
    app.middleware(RequestIdMiddleware)
        .middleware(LogRequest)
        .at("")
        .get(|_: Request<()>| async move { Ok(Response::new(StatusCode::NotFound)) })
        .all(|_: Request<()>| async move { Ok(Response::new(StatusCode::MethodNotAllowed)) })
        .at("/*")
        .get(|req: Request<()>| async move {
            let path = req.uri().path();
            let cache_control_header = HeaderName::from_str("cache-control").unwrap();
            // let expires_header = HeaderName::from_str("expires").unwrap();
            // let etag_header = HeaderName::from_str("etag").unwrap();
            // let last_modified_header = HeaderName::from_str("last-modified").unwrap();

            match path {
                "/version" => Ok(Response::new(StatusCode::Ok)
                    .set_header(cache_control_header, "no-store, max-age=0, s-maxage=0")
                    .body_json(&VersionResponse::new())?),
                "/health" => Ok(Response::new(StatusCode::Ok)
                    .set_header(cache_control_header, "no-store, max-age=0, s-maxage=0")
                    .body_json(&HealthResponse {
                        status: String::from("ok"),
                    })?),
                _ => Ok(Response::new(StatusCode::NotFound)),
            }
        })
        .all(|_: Request<()>| async move { Ok(Response::new(StatusCode::MethodNotAllowed)) });

    log::info!("Application starting on port: {}", port);

    app.listen(format!("0.0.0.0:{}", port)).await
}
