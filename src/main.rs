// response::Jsonを追加
use axum::{extract::{Path, State}, response::Json, routing::{get, patch}, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions}, types::chrono};
use std::env;

// JSONファイルのやりとりを可能にする
#[derive(Debug)]
// 構造体を定義
struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    pub created_at: chrono::NaiveDate,
}

// #[derive(Clone, Deserialize, Serialize)]
// struct Users {
//     users: Vec<User>,
// }

#[derive(Clone, Deserialize, Serialize)]
struct CreateUser {
    name: String,
}

// Readを行う関数
// usersを取得、Json<Users>を返り値として指定
async fn get_users(State(users_state): State<Arc<Mutex<Users>>>) -> Json<Users> {
    // ロックを獲得
    let user_lock = users_state.lock().await;
    // usersを返す
    Json(user_lock.clone())
}

// Createを行う関数
async fn post_user(
    State(users_state): State<Arc<Mutex<Users>>>,
    // bodyから受け取る値を書く
    create_user: Json<CreateUser>,
    ) -> Json<Users> {
        let mut users_lock = users_state.lock().await;

        // 追加するuserデータを定義
        let new_user = User {

            // usersの数から+1の値をu32としてidを定義
            id : (users_lock.users.len() + 1) as u32,

            // bodyで受け取ったJSONのnameを取得
            name: create_user.name.to_string(),
        };

        // usersに追加
        users_lock.users.push(new_user);

        // 更新されたusersを返す
        Json(users_lock.clone())
    }

async fn patch_user(
    State(users_state): State<Arc<Mutex<Users>>>,
    // URLから受け取る
    Path(user_id): Path<u32>,
    Json(update_user): Json<CreateUser>,
    // Resultを返り値に指定
    ) -> Result<Json<User>, String> {
    
    let mut users_lock = users_state.lock().await;

    // findの返り値がSome(user)であった場合のみ処理
    if let Some(user) = users_lock.users.iter_mut().find(|user| user.id == user_id) {

        // 名前を更新
        user.name = update_user.name.clone();

        // 値をOkに包んで返す
        return Ok(Json(user.clone()));
    }

    // idを持つuserが密じゃらなかったらErr()として返す
    Err("User not found".to_string())

}

async fn delete_user(
    State(users_state): State<Arc<Mutex<Users>>>,
    Path(user_id): Path<u32>,
    ) -> Result<Json<Users>, String> {
    
    let mut users_lock = users_state.lock().await;

    // 更新前のusersの長さを保持
    let original_len = users_lock.users.len();

    // retainを使って、指定したIDのユーザを削除
    users_lock.users.retain(|user| user.id != user_id) ;

    //usersの長さが変わっていれば、削除に成功している
    if users_lock.users.len() == original_len {
        Err("User not found".to_string())
    } else {
        Ok(Json(users_lock.clone()))
    }
}

/// Database object encapsulating the connection pool and providing convenience functions.
struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_options = SqliteConnectOptions::from_str(":memory:")?
            .create_if_missing(true)
            .disable_statement_logging()
            .to_owned();

        let pool = SqlitePoolOptions::new().connect_with(db_options).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                content NOT NULL
            );",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

// 非同期のmain関数を実行できるようにする
#[tokio::main]
async fn main() {
    // Hello Worldと返すハンドラーを定義
    async fn root_handler() -> String {
        "Hello World".to_string()
    }

    // let users = Users {
    //     users: vec![
    //         User {
    //             id: 1,
    //             name: "takashi".to_string(),
    //         },
    //         User {
    //             id: 2,
    //             name: "hitoshi".to_string(),
    //         },
    //         User {
    //             id: 3,
    //             name: "masashi".to_string(),
    //         },
    //     ],
    // };
    dotenv::dotenv().expect("Failed to read .env file");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&database_url).await;
    let users = sqlx::query_as!(
        User,
        "select id, name, email, address, created_at from users"
    ).fetch_all(&pool)
    .await;

    // Mutexにusersを包む。MutexをArcで包むのはイディオムのようなもの
    let users_state = Arc::new(Mutex::new(users));

    // ルートを定義
    // "/"を踏むと、上で定義したroot_handlerを実行する
    let app = Router::new()
                        .route("/users",get(get_users).post(post_user))
                        .route("/", get(root_handler))
                        .route("/users/:user_id", patch(patch_user).delete(delete_user))
                        .with_state(users_state);


    // 指定したポートにサーバを開く
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
