// Copyright 2022 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use axum::{
    http::header::{HeaderMap, HeaderName, HeaderValue},
    response::{IntoResponse, Json},
};
use http::StatusCode;
use serde_derive::Serialize;
use std::env;

#[derive(Debug, Serialize)]
struct VersionResponse {
    commit: String,
}

impl VersionResponse {
    fn new() -> Self {
        let commit = env::var("SOURCE_COMMIT").unwrap_or_else(|_| String::new());

        Self { commit }
    }
}

#[allow(clippy::unused_async)]
pub async fn get() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("no-store, max-age=0, s-maxage=0"),
    );
    let body = VersionResponse::new();
    (StatusCode::OK, headers, Json(body))
}
