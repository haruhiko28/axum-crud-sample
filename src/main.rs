// response::Jsonを追加
use axum::{http::StatusCode, extract::Query,extract::State, response::Json, routing::get, Extension, Router, response::IntoResponse};
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
    // pub address: Option<String>,
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

async fn patch_user(Query(query):Query<UserQuery>,Extension(pool):Extension<Arc<SqlitePool>>, Json(post): Json<UpdateUser>) -> impl IntoResponse {
    let selected_user_id: i64 = query.id;
    match sqlx::query!("UPDATE users SET name = ?, email = ? where id = ?;",
        post.name,
        post.email,
        selected_user_id)
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


// Createを行う関数
// async fn post_user(
//     State(users_state): State<Arc<Mutex<Users>>>,
//     // bodyから受け取る値を書く
//     create_user: Json<CreateUser>,
//     ) -> Json<Users> {
//         let mut users_lock = users_state.lock().await;

//         // 追加するuserデータを定義
//         let new_user = User {

//             // usersの数から+1の値をu32としてidを定義
//             id : (users_lock.users.len() + 1) as u32,

//             // bodyで受け取ったJSONのnameを取得
//             name: create_user.name.to_string(),
//         };

//         // usersに追加
//         users_lock.users.push(new_user);

//         // 更新されたusersを返す
//         Json(users_lock.clone())
//     }

// async fn patch_user(
//     State(users_state): State<Arc<Mutex<Users>>>,
//     // URLから受け取る
//     Path(user_id): Path<u32>,
//     Json(update_user): Json<CreateUser>,
//     // Resultを返り値に指定
//     ) -> Result<Json<User>, String> {
    
//     let mut users_lock = users_state.lock().await;

//     // findの返り値がSome(user)であった場合のみ処理
//     if let Some(user) = users_lock.users.iter_mut().find(|user| user.id == user_id) {

//         // 名前を更新
//         user.name = update_user.name.clone();

//         // 値をOkに包んで返す
//         return Ok(Json(user.clone()));
//     }

//     // idを持つuserが密じゃらなかったらErr()として返す
//     Err("User not found".to_string())

// }

// async fn delete_user(
//     State(users_state): State<Arc<Mutex<Users>>>,
//     Path(user_id): Path<u32>,
//     ) -> Result<Json<Users>, String> {
    
//     let mut users_lock = users_state.lock().await;

//     // 更新前のusersの長さを保持
//     let original_len = users_lock.users.len();

//     // retainを使って、指定したIDのユーザを削除
//     users_lock.users.retain(|user| user.id != user_id) ;

//     //usersの長さが変わっていれば、削除に成功している
//     if users_lock.users.len() == original_len {
//         Err("User not found".to_string())
//     } else {
//         Ok(Json(users_lock.clone()))
//     }
// }

/// Database object encapsulating the connection pool and providing convenience functions.
// struct Database {
//     pool: SqlitePool,
// }

// impl Database {
//     pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
//         let db_options = SqliteConnectOptions::from_str(":memory:")?
//             .create_if_missing(true)
//             .disable_statement_logging()
//             .to_owned();

//         let pool = SqlitePoolOptions::new().connect_with(db_options).await?;

//         sqlx::query(
//             "CREATE TABLE IF NOT EXISTS posts (
//                 id INTEGER PRIMARY KEY,
//                 title TEXT NOT NULL,
//                 content NOT NULL
//             );",
//         )
//         .execute(&pool)
//         .await?;

//         Ok(Self { pool })
//     }
// }

// 非同期のmain関数を実行できるようにする
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // Hello Worldと返すハンドラーを定義
    // async fn root_handler() -> String {
    //     "Hello World".to_string()
    // }

    dotenv::dotenv().expect("Failed to read .env file");
    let key = "DATABASE_URL";
    let db_url = env::var(key).expect("key not found.");
    let pool = SqlitePool::connect(&db_url).await.expect("cannot connect.");
    let shared_pool = Arc::new(pool);
   
    // Mutexにusersを包む。MutexをArcで包むのはイディオムのようなもの
    // let users_state = Arc::new(Mutex::new(users));

    // ルートを定義
    // "/"を踏むと、上で定義したroot_handlerを実行する
    // let app = Router::new()
                        // .route("/users",get(get_users).post(post_user))
                        // .route("/users",get(get_users))

    //                     .route("/", get(root_handler))
                        // .route("/users/:user_id", patch(patch_user).delete(delete_user))
                        // .with_state(users_state);
    let app = Router::new()
        .route("/users", get(users_handler).post(add_user))
        .route("/user", get(user_handler))
        .layer(Extension(shared_pool));

    // 指定したポートにサーバを開く
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
