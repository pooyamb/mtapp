use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use axum::body::{self, BoxBody, Bytes, HttpBody};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::{Request, Response};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{BoxError, Extension, TypedHeader};
use basteh::Storage;
use tower::{Layer, Service};

use crate::app::AuthConfig;
use crate::errors::AuthError;
use crate::extract::Claims;

pub async fn jwt_claims<B>(
    config: Extension<AuthConfig>,
    storage: Extension<Storage>,
    token: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request<B>,
    next: Next<B>,
) -> axum::response::Response {
    if let Some(token) = token {
        if request.extensions().get::<Claims>().is_none() {
            // try to extract the claims from header token
            let claims = match Claims::from_token(token.token(), config.expose_secret()) {
                Ok(val) => val,
                Err(_) => {
                    return (AuthError::BadToken).into_response();
                }
            };

            let blacklisted = match storage
                .scope(config.blacklist_scope())
                .contains_key(claims.inner().jti)
                .await
            {
                Ok(a) => a,
                // Internal error
                Err(e) => return (AuthError::StorageError(e)).into_response(),
            };

            if blacklisted {
                return (AuthError::BadToken).into_response();
            }

            request.extensions_mut().insert(claims.clone());
        }
    }

    next.run(request).await
}

#[derive(Clone)]
pub struct ClaimCheck<F> {
    check_fn: Arc<F>,
}

impl<F> ClaimCheck<F> {
    pub fn new(check_fn: F) -> Self {
        Self {
            check_fn: Arc::new(check_fn),
        }
    }
}

impl<F, T> Layer<T> for ClaimCheck<F> {
    type Service = ClaimCheckService<F, T>;

    fn layer(&self, inner: T) -> Self::Service {
        ClaimCheckService {
            check_fn: self.check_fn.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct ClaimCheckService<F, S> {
    check_fn: Arc<F>,
    inner: S,
}

impl<ReqBody, ResBody, F, S> Service<Request<ReqBody>> for ClaimCheckService<F, S>
where
    F: Fn(Option<Claims>) -> bool,
    F: Send + Sync + 'static,
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = Infallible>,
    S::Future: Send + 'static,
    ResBody: HttpBody<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError>,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let check_fn = self.check_fn.clone();
        let claims = req.extensions().get().cloned();

        let fut = self.inner.call(req);

        Box::pin(async move {
            if !check_fn(claims) {
                Ok(AuthError::Permission.into_response())
            } else {
                Ok(fut.await?.map(body::boxed))
            }
        })
    }
}
