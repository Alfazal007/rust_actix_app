use actix_web::{
    cookie::{Cookie, SameSite},
    web, HttpResponse, Responder,
};
use serde::Serialize;
use validator::Validate;

use crate::{errors::GeneralError, models::UserCreateToDB, AppState};

#[derive(Serialize)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
}

pub async fn login(
    pool: web::Data<AppState>,
    new_user: web::Json<UserCreateToDB>,
) -> impl Responder {
    if let Err(e) = new_user.validate() {
        let mut validation_errors: Vec<String> = Vec::new();
        for (_, err) in e.field_errors().iter() {
            if let Some(message) = &err[0].message {
                validation_errors.push(message.clone().into_owned());
            }
        }
        return HttpResponse::BadRequest().json(crate::errors::ValidationErrorsToBeReturned {
            errors: validation_errors,
        });
    }

    let user_from_db =
        sqlx::query_as::<_, crate::models::UserToLogin>("SELECT * FROM users where username = $1")
            .bind(new_user.0.username)
            .fetch_optional(&pool.db)
            .await;

    if user_from_db.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            errors: "Issue talking to the database".to_string(),
        });
    }

    if user_from_db.as_ref().unwrap().is_none() {
        return HttpResponse::InternalServerError().json(GeneralError {
            errors: "User with this username not found".to_string(),
        });
    }

    let user = user_from_db.unwrap().unwrap();

    let verify = bcrypt::verify(new_user.0.password, &user.password);
    if verify.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            errors: "Issue validating the password".to_string(),
        });
    }

    if !verify.unwrap() {
        return HttpResponse::InternalServerError().json(GeneralError {
            errors: "Invalid password".to_string(),
        });
    }

    let token =
        crate::tokens::generate_token::generate_token(&user.username, user.id, &pool.access_secret);

    if token.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            errors: "Issue generating the token".to_string(),
        });
    }

    let cookie = Cookie::build("accessToken", token.as_ref().unwrap())
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::None)
        .finish();

    HttpResponse::Ok().cookie(cookie).json(LoginResponse {
        access_token: token.unwrap(),
    })
}
