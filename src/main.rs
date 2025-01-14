use std::env;

use actix_web::{
    middleware::from_fn,
    web::{self, Data},
    App, HttpServer,
};
use dotenvy::dotenv;
use middleware::auth_middleware;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod errors;
mod middleware;
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

    let database_url = env::var("DATABASE_URL").expect("Database url not found in env files");
    let secret_key = env::var("ACCESSTOKENSECRET").expect("Secret key not found in env files");

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
                    .service(
                        web::scope("/protected")
                            .wrap(from_fn(auth_middleware::auth_middleware))
                            .route(
                                "/currentUser",
                                web::get().to(routes::user::current_user::get_current_user),
                            ),
                    ),
            )
    })
    .bind(("127.0.0.1", 8000))?
    .run();

    println!("The server started at port 8000");
    server.await
}
