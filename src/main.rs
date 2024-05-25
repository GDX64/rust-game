use std::{
    ops::Add,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
mod game;

type AppState = Arc<Mutex<game::TrucoApp>>;

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(game::TrucoApp::new()));
    // build our application with a single route
    let app = Router::new()
        .route("/new_user", get(new_user_handler))
        .route("/new_room", get(new_room_handler))
        .route("/rooms", get(rooms_handler))
        .route("/ws", get(ws_handler))
        .route("/add_user_to_room", get(add_user_to_room_handler))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    let res = ws.on_upgrade(|ws| {
        async {
            let (mut send, mut receive) = ws.split();
            loop {
                let msg = receive.next().await;
                match msg {
                    Some(Ok(msg)) => {
                        println!("{:?}", msg);
                        send.send(msg).await.unwrap();
                    }
                    _ => break,
                }
            }
        }
    });
    res
}

#[derive(Deserialize)]
struct UserParms {
    name: String,
}

async fn new_user_handler(params: Query<UserParms>, state: State<AppState>) -> impl IntoResponse {
    if let Ok(mut lock) = state.0.lock() {
        let player = lock.create_player(&params.name);
        Json(player).into_response()
    } else {
        Json("").into_response()
    }
}

#[derive(Deserialize)]
struct RoomParms {
    name: String,
}

async fn new_room_handler(params: Query<RoomParms>, state: State<AppState>) -> impl IntoResponse {
    if let Ok(mut lock) = state.0.lock() {
        lock.create_room(&params.name);
    }
}

async fn rooms_handler(state: State<AppState>) -> impl IntoResponse {
    if let Ok(lock) = state.0.lock() {
        let rooms = lock.get_rooms();
        Json(rooms).into_response()
    } else {
        Json(()).into_response()
    }
}

#[derive(Deserialize)]
struct AddUserToRoomParms {
    room: String,
    user_id: u64,
}

async fn add_user_to_room_handler(
    params: Query<AddUserToRoomParms>,
    state: State<AppState>,
) -> impl IntoResponse {
    if let Ok(mut lock) = state.0.lock() {
        let player = lock.add_user_to_room(&params.room, params.user_id);
        Json(player).into_response()
    } else {
        Json("").into_response()
    }
}
