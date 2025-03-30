// response::Jsonを追加
use axum::{extract::State, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// JSONファイルのやりとりを可能にする
#[derive(Clone, Deserialize, Serialize)]
// 構造体を定義
struct User {
    id: u32,
    name: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Users {
    users: Vec<User>,
}

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

// 非同期のmain関数を実行できるようにする
#[tokio::main]
async fn main() {
    // Hello Worldと返すハンドラーを定義
    async fn root_handler() -> String {
        "Hello World".to_string()
    }

    let users = Users {
        users: vec![
            User {
                id: 1,
                name: "takashi".to_string(),
            },
            User {
                id: 2,
                name: "hitoshi".to_string(),
            },
            User {
                id: 3,
                name: "masashi".to_string(),
            },
        ],
    };

    // Mutexにusersを包む。MutexをArcで包むのはイディオムのようなもの
    let users_state = Arc::new(Mutex::new(users));

    // ルートを定義
    // "/"を踏むと、上で定義したroot_handlerを実行する
    let app = Router::new()
                        .route("/users",get(get_users).post(post_user))
                        .route("/", get(root_handler))
                        .with_state(users_state);


    // 指定したポートにサーバを開く
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
