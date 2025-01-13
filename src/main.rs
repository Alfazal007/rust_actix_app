use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod models;
mod routes;

pub struct AppState {
    pub db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://postgres:mysecretpassword@localhost:5432/postgres")
        .await
        .expect("Issue creating the connection pool");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .service(routes::init::hello)
            .service(web::scope("/api/v1/user").route(
                "/get/{id}",
                web::get().to(routes::user::create_user::get_user_by_id),
            ))
    })
    .bind(("127.0.0.1", 8000))?
    .run();
    println!("The server started at port 8000");
    server.await
}
