use crate::{
    errors::GeneralError,
    middleware::auth_middleware::UserData,
    models::{TodoCreateToDB, TodoFromDB},
    AppState,
};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use validator::Validate;

pub async fn create_todo(
    req: HttpRequest,
    app_data: web::Data<AppState>,
    new_todo: web::Json<TodoCreateToDB>,
) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue talking to the database".to_string(),
        });
    }
    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    if let Err(e) = new_todo.validate() {
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

    let result = sqlx::query_as::<_, TodoFromDB>(
        "INSERT INTO todos(creator_id, title) VALUES ($1, $2) RETURNING *",
    )
    .bind(user_data.user_id)
    .bind(&new_todo.title)
    .fetch_optional(&app_data.db)
    .await;

    println!("{:?}", result);

    if result.is_err() {
        return HttpResponse::BadRequest().json(GeneralError {
            errors: "Issue writing to the database".to_string(),
        });
    }

    if result.as_ref().unwrap().is_none() {
        return HttpResponse::BadRequest().json(GeneralError {
            errors: "Issuw writing to the database".to_string(),
        });
    }

    let todos_from_db = result.unwrap().unwrap();

    HttpResponse::Ok().json(todos_from_db)
}
