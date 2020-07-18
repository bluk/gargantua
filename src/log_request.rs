// Copyright 2020 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use tide::{http::headers, Middleware, Next, Request};

use crate::request_id::RequestId;

/// Logs the request.
#[derive(Debug, Default)]
pub struct LogRequest;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for LogRequest {
    async fn handle(&self, ctx: Request<State>, next: Next<'_, State>) -> tide::Result {
        let host = ctx
            .header(&headers::HOST)
            .map(|vals| {
                vals.iter()
                    .map(|v| String::from(v.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| String::from(""));
        let origin = ctx
            .header(&headers::ORIGIN)
            .map(|vals| {
                vals.iter()
                    .map(|v| String::from(v.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| String::from(""));
        let method = ctx.method().to_string();
        let path = ctx.url().path().to_owned();
        let req_id = ctx
            .ext::<RequestId>()
            .map(|id| id.0.clone())
            .unwrap_or_else(|| String::from(""));
        let start = std::time::Instant::now();

        let res = next.run(ctx).await;
        let status = res.status();
        if status.is_server_error() {
            if let Some(err) = res.error() {
                log::info!(
                    "host={} origin={} method={} path={} error_msg={} status={} elapsed={} request_id={}",
                    host,
                    origin,
                    method,
                    path,
                    err.to_string(),
                    status as u16,
                    start.elapsed().as_millis(),
                    req_id
                );
            } else {
                log::info!(
                    "host={} origin={} method={} path={} status={} elapsed={} request_id={}",
                    host,
                    origin,
                    method,
                    path,
                    status as u16,
                    start.elapsed().as_millis(),
                    req_id
                );
            }
        } else {
            log::info!(
                "host={} origin={} method={} path={} status_code={} elapsed={} request_id={}",
                host,
                origin,
                method,
                path,
                res.status(),
                start.elapsed().as_millis(),
                req_id
            );
        }

        Ok(res)
    }
}
