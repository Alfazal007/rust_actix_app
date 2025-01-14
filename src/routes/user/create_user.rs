use actix_web::{web, HttpResponse, Responder};
use validator::Validate;

use crate::{models::UserCreateToDB, AppState};

pub async fn create_user(
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

    // check database for same username
    let user_with_same_username =
        sqlx::query_as::<_, crate::models::UserFromDB>("SELECT * FROM users WHERE username = $1")
            .bind(&new_user.0.username)
            .fetch_optional(&pool.db)
            .await;

    match user_with_same_username {
        Err(_) => {
            return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
                errors: "Issue talking to the database".to_string(),
            });
        }
        Ok(data) => {
            if data.is_some() {
                return HttpResponse::BadRequest().json(crate::errors::GeneralError {
                    errors: "User with this username already exists".to_string(),
                });
            }
        }
    }

    // hash password
    let hashed_password = bcrypt::hash(new_user.0.password, 12);
    if hashed_password.is_err() {
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue hashing the password".to_string(),
        });
    }

    let hased_password_string = hashed_password.unwrap();

    let new_user = sqlx::query_as::<_, crate::models::UserFromDB>(
        "INSERT INTO users(username, password) VALUES ($1, $2) returning *",
    )
    .bind(new_user.0.username)
    .bind(hased_password_string)
    .fetch_optional(&pool.db)
    .await;

    if new_user.is_err() {
        println!("{:?}", new_user);
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue writing to the database".to_string(),
        });
    }

    if new_user.as_ref().unwrap().is_none() {
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue writing to the database".to_string(),
        });
    }

    HttpResponse::Ok().json(new_user.unwrap())
}
