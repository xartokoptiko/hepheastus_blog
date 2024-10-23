use actix_web::{dev::ServiceRequest, Error, HttpMessage, HttpResponse};
use futures_util::future::{ok, Ready};
use std::task::{Context, Poll};
use std::pin::Pin;
use actix_web::dev::{Service, Transform};
use futures_util::TryFutureExt;
use crate::utils;
use utils::{validate_jwt};

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service })
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = actix_web::dev::ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = actix_web::dev::ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn futures::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token_opt = req.headers()
            .get("Authorization")
            .and_then(|header_value| header_value.to_str().ok())
            .map(|bearer| bearer.trim_start_matches("Bearer ").to_string());

        if let Some(token) = token_opt {
            match validate_jwt(&token) {
                Ok(claims) => {
                    req.extensions_mut().insert(claims);
                    let fut = self.service.call(req);
                    Box::pin(async move { fut.await })
                }
                Err(_) => {
                    let unauthorized_response = HttpResponse::Unauthorized().finish();
                    let res: actix_web::dev::ServiceResponse<B> = req.into_response(HttpResponse::Unauthorized().finish().into_body());
                    Box::pin(async move { Ok(res) })
                }
            }
        } else {
            let res: actix_web::dev::ServiceResponse<B> = req.into_response(HttpResponse::Unauthorized().finish().into_body());
            Box::pin(async move { Ok(res) })
        }
    }
}
