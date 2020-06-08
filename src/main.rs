// Copyright 2020 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use http_types::headers;
use serde::Serialize;
use std::{
    env,
    time::{Duration, SystemTime},
};
use tide::{Body, Request, Response, StatusCode};

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
            let path = req.url().path();

            match path {
                "/version" => {
                    let mut resp = Response::new(StatusCode::Ok);
                    resp.insert_header(headers::CACHE_CONTROL, "no-store, max-age=0, s-maxage=0");
                    resp.set_body(Body::from_json(&VersionResponse::new())?);
                    Ok(resp)
                }
                "/health" => {
                    let mut resp = Response::new(StatusCode::Ok);
                    resp.insert_header(headers::CACHE_CONTROL, "no-store, max-age=0, s-maxage=0");
                    resp.set_body(Body::from_json(&HealthResponse {
                        status: String::from("ok"),
                    })?);
                    Ok(resp)
                }
                _ => {
                    let mut resp = Response::new(StatusCode::NotFound);
                    resp.insert_header(
                        headers::CACHE_CONTROL,
                        "public, max-age=21600, s-maxage=21600, must-revalidate",
                    );
                    let now = SystemTime::now();
                    resp.insert_header(headers::LAST_MODIFIED, httpdate::fmt_http_date(now));
                    let expires = now + Duration::from_secs(60 * 60);
                    resp.insert_header(headers::EXPIRES, httpdate::fmt_http_date(expires));
                    Ok(resp)
                }
            }
        })
        .all(|_: Request<()>| async move { Ok(Response::new(StatusCode::MethodNotAllowed)) });

    log::info!("Application starting on port: {}", port);

    app.listen(format!("0.0.0.0:{}", port)).await
}
