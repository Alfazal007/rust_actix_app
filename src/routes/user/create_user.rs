use actix_web::{web, Responder};

use crate::AppState;

pub async fn get_user_by_id(state: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    println!("The id is {:?}", id);
    let user_from_id =
        sqlx::query_as::<_, crate::models::UserFromDB>("SELECT * FROM users WHERE id = $1")
            .bind(id.into_inner())
            .fetch_optional(&state.db)
            .await;
    match user_from_id {
        Err(e) => {
            println!("There was an issue fetching {:?}", e);
        }
        Ok(data) => match data {
            Some(user) => {
                println!("The user is {:?}", user)
            }
            None => {
                println!("User not found")
            }
        },
    }
    "hello"
}
