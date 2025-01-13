use crate::tokens::generate_token::Claims;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    web::Data,
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use sqlx::PgPool;
use std::env;

#[derive(Clone)]
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let token = match extract_token(&req) {
            Ok(token) => token,
            Err(e) => return Box::pin(async move { Err(e) }),
        };

        let secret = match env::var("ACCESSTOKENSECRET") {
            Ok(s) => s,
            Err(_) => {
                return Box::pin(
                    async move { Err(ErrorUnauthorized("Server configuration error")) },
                )
            }
        };

        let claims = match validate_token(&token, &secret) {
            Ok(claims) => claims,
            Err(e) => return Box::pin(async move { Err(e) }),
        };

        // Get database pool from app data
        let pool = req.app_data::<Data<PgPool>>().cloned();
        if pool.is_none() {
            return Box::pin(async move { Err(ErrorUnauthorized("Database configuration error")) });
        }

        let pool = pool.unwrap();
        let user_id = claims.user_id;
        let username = claims.username.clone();

        let service = self.service.clone();

        Box::pin(async move {
            match verify_user_exists(&pool, user_id, username).await {
                Ok(true) => {
                    req.extensions_mut().insert(claims);
                    let res = service.call(req).await?;
                    Ok(res)
                }
                Ok(false) => Err(ErrorUnauthorized("User not found in database")),
                Err(_) => Err(ErrorUnauthorized("Database error during user verification")),
            }
        })
    }
}

fn extract_token(req: &ServiceRequest) -> Result<String, Error> {
    if let Some(cookie) = req.cookie("accessToken") {
        Ok(cookie.value().to_string())
    } else {
        Err(ErrorUnauthorized("No accessToken cookie found"))
    }
}

fn validate_token(token: &str, secret: &str) -> Result<Claims, Error> {
    let validation = Validation::new(Algorithm::HS256);
    let key = DecodingKey::from_secret(secret.as_bytes());
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(err) => match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                Err(ErrorUnauthorized("Token expired"))
            }
            jsonwebtoken::errors::ErrorKind::InvalidToken => {
                Err(ErrorUnauthorized("Invalid token"))
            }
            _ => Err(ErrorUnauthorized("Token validation failed")),
        },
    }
}

async fn verify_user_exists(
    pool: &PgPool,
    user_id: i32,
    username: String,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND username = $2) as exists",
        user_id,
        username
    )
    .fetch_one(pool)
    .await?;

    Ok(result.exists.unwrap_or(false))
}
