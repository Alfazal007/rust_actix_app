#[derive(sqlx::FromRow, Debug)]
pub struct UserFromDB {
    username: String,
    id: i32,
    password: String,
}
