use crate::models::TodoFromDB;
use crate::{middleware::auth_middleware::UserData, AppState};
use actix_web::{HttpMessage, HttpResponse};
use validator::Validate;

use actix_web::{web, HttpRequest, Responder};

#[derive(serde::Deserialize, Validate)]
pub struct TodoId {
    #[validate(range(min = 1, message = "Todo id not provided"))]
    id: i32,
}

pub async fn mark_done_todo(
    req: HttpRequest,
    app_data: web::Data<AppState>,
    todo: web::Json<TodoId>,
) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue talking to the database".to_string(),
        });
    }
    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    if let Err(e) = todo.validate() {
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

    let todo_existing =
        sqlx::query_as::<_, TodoFromDB>("select * from todos where id=$1 and creator_id=$2")
            .bind(todo.id)
            .bind(user_data.user_id)
            .fetch_optional(&app_data.db)
            .await;

    if todo_existing.is_err() {
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue talking to the database".to_string(),
        });
    }

    if todo_existing.as_ref().unwrap().is_none() {
        return HttpResponse::NotFound().json(crate::errors::GeneralError {
            errors: "Todo not found".to_string(),
        });
    }

    if todo_existing.unwrap().unwrap().completed {
        return HttpResponse::Ok().json(());
    }

    let updated_existing = sqlx::query_as::<_, TodoFromDB>(
        "update todos set completed=true where id=$1 and creator_id=$2 returning *",
    )
    .bind(todo.id)
    .bind(user_data.user_id)
    .fetch_optional(&app_data.db)
    .await;

    if updated_existing.is_err() {
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue talking to the database".to_string(),
        });
    }

    if updated_existing.as_ref().unwrap().is_none() {
        return HttpResponse::NotFound().json(crate::errors::GeneralError {
            errors: "Todo not found".to_string(),
        });
    }

    HttpResponse::Ok().json(updated_existing.unwrap().unwrap())
}
