use actix_web::Responder;

pub async fn get_current_user() -> impl Responder {
    "hello"
}
