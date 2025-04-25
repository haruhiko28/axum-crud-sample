// response::Jsonを追加
use axum::{http::StatusCode, extract::Query,extract::Path, response::Json, routing::get,routing::patch, Extension, Router, response::IntoResponse};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{sqlite::SqlitePool, types::chrono};
use std::env;
use serde_json::json;

// JSONファイルのやりとりを可能にする
#[derive(Serialize, sqlx::FromRow)]
struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    // pub created_at: chrono::NaiveDate,
}

#[derive(Deserialize)]
struct UserQuery {
    id: i64,
}

async fn user_handler(Query(query):Query<UserQuery>, Extension(pool):Extension<Arc<SqlitePool>>) -> impl IntoResponse {
    let selected_user_id = query.id;
    match sqlx::query_as!(User, "select id, name, email, address from users where id = ?", selected_user_id).fetch_optional(&*pool).await {
        Ok(user) => (StatusCode::OK, Json(user)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None::<User>)),
    }
}

async fn users_handler(Extension(pool):Extension<Arc<SqlitePool>>) -> impl IntoResponse {
    match  sqlx::query_as::<_, User>("select * from users")
        .fetch_all(&*pool)
        .await {
            Ok(users) => {
                let json_users = json!(users); // Vec<User> を JSON に変換
                (StatusCode::OK, Json(json_users))
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            ),
    // return (StatusCode::OK, Json(users));
    }
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Serialize)]
struct InsertResponse {
    rows_affected: u64,
}

#[derive(Deserialize)]
pub struct UpdateUser {
    name: String,
    email: String,
}

async fn add_user(Extension(pool):Extension<Arc<SqlitePool>>, Json(post): Json<CreateUser>) -> impl IntoResponse {
    match sqlx::query!("INSERT INTO users (name, email) VALUES (?,?);",
    post.name,
    post.email)
    .execute(&*pool)
    .await {
        Ok(result) => {
            let response = InsertResponse {
                rows_affected: result.rows_affected(),
            };
            let json_users = json!(response); // Vec<User> を JSON に変換
            (StatusCode::OK, Json(json_users))
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))
    }
}

async fn patch_user(Path(user_id): Path<u32>, Extension(pool):Extension<Arc<SqlitePool>>, Json(post): Json<UpdateUser>) -> impl IntoResponse {
    // let selected_user_id: i64 = query.id;
    match sqlx::query!("UPDATE users SET name = ?, email = ? where id = ?;",
        post.name,
        post.email,
        user_id)
        .execute(&*pool)
        .await {
            Ok(result) => {
                let response = InsertResponse {
                    rows_affected: result.rows_affected(),
                };
                let json_users = json!(response); // Vec<User> を JSON に変換
                (StatusCode::OK, Json(json_users))
            },
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))
    }
}
async fn delete_user(Path(user_id): Path<u32>, Extension(pool):Extension<Arc<SqlitePool>>, Json(post): Json<UpdateUser>) -> impl IntoResponse {
    match sqlx::query!("DELETE FROM users where id = ?;", 
        user_id)
        .execute(&*pool)
        .await {
            Ok(result) => {
                let response = InsertResponse {
                    rows_affected: result.rows_affected(),
                };
                let json_users = json!(response); // Vec<User> を JSON に変換
                (StatusCode::OK, Json(json_users))
            },
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))
        }
}

// 非同期のmain関数を実行できるようにする
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {

    dotenv::dotenv().expect("Failed to read .env file");
    let key = "DATABASE_URL";
    let db_url = env::var(key).expect("key not found.");
    let pool = SqlitePool::connect(&db_url).await.expect("cannot connect.");
    let shared_pool = Arc::new(pool);
   
    let app = Router::new()
        .route("/users", get(users_handler).post(add_user))
        .route("/user", get(user_handler))
        .route("/users/:user_id", patch(patch_user).delete(delete_user))
        .layer(Extension(shared_pool));

    // 指定したポートにサーバを開く
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
