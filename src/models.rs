use serde::Serialize;
use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct UserCreateToDB {
    #[validate(length(
        min = 6,
        max = 20,
        message = "Username should be between 6 and 20 length"
    ))]
    pub username: String,
    #[validate(length(
        min = 6,
        max = 20,
        message = "Password should be between 6 and 20 length"
    ))]
    pub password: String,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct UserFromDB {
    pub username: String,
    pub id: i32,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct UserToLogin {
    pub username: String,
    pub id: i32,
    pub password: String,
}
