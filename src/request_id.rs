// Copyright 2020 Bryant Luk
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str::FromStr;
use tide::{http::headers::HeaderName, Middleware, Next, Request};
use uuid::Uuid;

/// Identifies a request.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RequestId(pub String);

/// Sets a request ID in the local context and as a response header.
#[derive(Debug, Default)]
pub struct RequestIdMiddleware;

#[async_trait::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for RequestIdMiddleware {
    async fn handle(&self, mut ctx: Request<State>, next: Next<'_, State>) -> tide::Result {
        let req_id = Uuid::new_v4().to_string();

        ctx.set_ext::<RequestId>(RequestId(req_id.clone()));

        let req_id_header = HeaderName::from_str("request-id").unwrap();

        let mut res = next.run(ctx).await;
        res.insert_header(req_id_header, req_id.clone());
        Ok(res)
    }
}
