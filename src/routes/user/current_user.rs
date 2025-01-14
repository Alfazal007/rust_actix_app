use crate::{middleware::auth_middleware::UserData, AppState};
use actix_web::{HttpMessage, HttpResponse};

use actix_web::{web, HttpRequest, Responder};

pub async fn get_current_user(req: HttpRequest, _: web::Data<AppState>) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(crate::errors::GeneralError {
            errors: "Issue talking to the database".to_string(),
        });
    }
    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    HttpResponse::Ok().json(UserData {
        user_id: user_data.user_id,
        username: user_data.username.to_string(),
    })
}
