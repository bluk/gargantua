// Copyright 2020 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::future::Future;
use std::pin::Pin;
use tide::{http::headers, Middleware, Next, Request};

use crate::request_id::RequestId;

/// Logs the request.
#[derive(Debug, Default)]
pub struct LogRequest;

impl<State: Send + Sync + 'static> Middleware<State> for LogRequest {
    fn handle<'a>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
        Box::pin(async move {
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
            let path = ctx.uri().path().to_owned();
            let req_id = ctx
                .local::<RequestId>()
                .map(|id| id.0.clone())
                .unwrap_or_else(|| String::from(""));
            let start = std::time::Instant::now();

            match next.run(ctx).await {
                Ok(res) => {
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
                    Ok(res)
                }
                Err(err) => {
                    log::info!(
                        "host={} origin={} method={} path={} error_msg={} elapsed={} request_id={}",
                        host,
                        origin,
                        method,
                        path,
                        err.to_string(),
                        start.elapsed().as_millis(),
                        req_id
                    );
                    Err(err)
                }
            }
        })
    }
}
