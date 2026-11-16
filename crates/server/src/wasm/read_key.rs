//! The Worker side of the verdicts read key (`crate::read_key`, #1352):
//! a signed-in device makes, checks or removes the key, and the content
//! tools read the marks with it.

use batna_protocol::cloud::{ErrorCode, NewReadKey};
use worker::{Env, Headers, Method, Request, RequestInit, Response, Result};

use super::{account_stub, api_error, bearer, call_do, now_ms};
use crate::internal::{READ_KEY_HEADER, ReadKeyHash};
use crate::tokens::{self, AccessClaims};

/// `GET`, `POST` or `DELETE /v1/verdicts/key` from a signed-in device.
pub(super) async fn manage(env: &Env, claims: &AccessClaims, method: Method) -> Result<Response> {
    match method {
        Method::Post => {
            let Ok(secret) = tokens::random_bytes() else {
                return api_error(ErrorCode::Internal, "no secure randomness for the key");
            };
            let (key, hash) = crate::read_key::new_key(&claims.sub, secret);
            let created_ms = now_ms();
            let resp = call_do(
                env,
                &claims.sub,
                Method::Post,
                "/read-key",
                Some(&claims.dev),
                Some(&ReadKeyHash { hash }),
            )
            .await?;
            if resp.status_code() != 200 {
                return Ok(resp);
            }
            Response::from_json(&NewReadKey { key, created_ms })
        }
        m => call_do(env, &claims.sub, m, "/read-key", Some(&claims.dev), None::<&()>).await,
    }
}

/// `GET /v1/verdicts` with the read key as the bearer token.
pub(super) async fn verdicts(req: &Request, env: &Env) -> Result<Response> {
    let Some((account, hash)) = bearer(req).as_deref().and_then(crate::read_key::parse) else {
        return api_error(ErrorCode::Unauthorized, "expected the verdicts read key");
    };
    let headers = Headers::new();
    headers.set(READ_KEY_HEADER, &hash)?;
    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);
    let forwarded = Request::new_with_init("https://account/verdicts", &init)?;
    account_stub(env, &account)?.fetch_with_request(forwarded).await
}
