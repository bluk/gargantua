// Copyright 2022 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use axum::{
    http::header::{HeaderMap, HeaderName, HeaderValue},
    response::IntoResponse,
};
use http::StatusCode;
use std::time::{Duration, SystemTime};

pub async fn get_not_found() -> impl IntoResponse {
    let now = SystemTime::now();
    let expires = now + Duration::from_secs(60 * 60);

    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("cache-control"),
        HeaderValue::from_static("public, max-age=21600, s-maxage=21600, must-revalidate"),
    );
    headers.insert(
        HeaderName::from_static("last-modified"),
        HeaderValue::from_str(&httpdate::fmt_http_date(now)).unwrap(),
    );
    headers.insert(
        HeaderName::from_static("expires"),
        HeaderValue::from_str(&httpdate::fmt_http_date(expires)).unwrap(),
    );
    (StatusCode::NOT_FOUND, headers)
}

pub async fn any_method_not_allowed() -> impl IntoResponse {
    StatusCode::METHOD_NOT_ALLOWED
}
