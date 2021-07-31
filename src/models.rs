#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct User {
    pub id: i64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub name: String,
    pub email: String,
}

#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct AuthToken {
    pub id: i64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub expires: chrono::DateTime<chrono::Utc>,
    // a user's (of this application) authentication token
    // that is set as cookie on a user's session. This value
    // stored is the hmac (and hex encoded) signature of the
    // actual token returned to the user.
    pub signature: String,
    pub user_id: i64,
}

#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct Map {
    pub id: i64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub deleted: bool,
    pub english: String,
    pub japanese: String,
}

#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct Question {
    pub id: i64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub user_id: i64,
    pub map_id: i64,
    pub prompt: String,
    pub answer: String,
    pub answered: Option<chrono::DateTime<chrono::Utc>>,
    pub correct: Option<bool>,
}
