// Copyright 2023 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;

use axum::response::{IntoResponse, Json};
use http::StatusCode;
use serde_json::{Map, Value};

#[allow(clippy::unused_async)]
pub async fn get() -> impl IntoResponse {
    let vars = env::vars();
    let mut body = Map::new();
    for (k, v) in vars {
        body.insert(k, Value::String(v));
    }
    (StatusCode::OK, Json(body))
}
