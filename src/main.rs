use std::env;

use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use authmiddleware::AuthMiddleware;
use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod authmiddleware;
mod errors;
mod models;
mod routes;
mod tokens;

pub struct AppState {
    pub db: Pool<Postgres>,
    access_secret: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").unwrap();
    let secret_key = env::var("ACCESSTOKENSECRET").unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Issue creating the connection pool");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                db: pool.clone(),
                access_secret: secret_key.clone(),
            }))
            .service(routes::init::hello)
            .service(
                web::scope("/api/v1/user")
                    .route("/signin", web::post().to(routes::user::login::login))
                    .route(
                        "/create",
                        web::post().to(routes::user::create_user::create_user),
                    )
                    .service(web::scope("/protected").wrap(AuthMiddleware).route(
                        "/currentUser",
                        web::get().to(routes::user::current_user::get_current_user),
                    )),
            )
    })
    .bind(("127.0.0.1", 8000))?
    .run();

    println!("The server started at port 8000");
    server.await
}
