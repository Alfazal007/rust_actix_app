use actix_web::{get, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    "init"
}
