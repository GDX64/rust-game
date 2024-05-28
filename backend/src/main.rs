use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use canvas_game::GameMessage;
use futures_util::StreamExt;
use serde::Deserialize;
use tokio::sync::oneshot;
mod canvas_game;
mod game;

struct Apps {
    truco: game::TrucoApp,
    canvas_tx: tokio::sync::mpsc::Sender<GameMessage>,
}

impl Apps {
    fn new(tx: tokio::sync::mpsc::Sender<GameMessage>) -> Apps {
        Apps {
            truco: game::TrucoApp::new(),
            canvas_tx: tx,
        }
    }
}

type AppState = Arc<Mutex<Apps>>;

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    tokio::spawn(canvas_game::CanvasGame::run(rx));
    let state: AppState = Arc::new(Mutex::new(Apps::new(tx)));
    // build our application with a single route
    let app = Router::new()
        .route("/new_user", get(new_user_handler))
        .route("/new_room", get(new_room_handler))
        .route("/", get(|| async { "Hello, World!" }))
        .route("/rooms", get(rooms_handler))
        .route("/ws", get(ws_handler))
        .route("/add_user_to_room", get(add_user_to_room_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, state: State<AppState>) -> impl IntoResponse {
    let tx = state.lock().unwrap().canvas_tx.clone();
    let res = ws.on_upgrade(move |ws| {
        async move {
            let (send, mut receive) = ws.split();
            let (send_id, receive_id) = oneshot::channel();
            let send_result = tx
                .send(GameMessage::NewConnection {
                    id_sender: send_id,
                    sender: send,
                })
                .await;

            if let Err(err) = send_result {
                println!("Error: {:?}", err)
            }

            let id = receive_id.await.unwrap();
            loop {
                let msg = receive.next().await;
                match msg {
                    Some(Ok(msg)) => {
                        let txt = msg.to_text().unwrap_or("msg is not a text");
                        let send_result =
                            tx.send(GameMessage::ClientMessage(txt.to_string())).await;
                        if let Err(err) = send_result {
                            println!("Error: {:?}", err)
                        }
                    }
                    _ => {
                        println!("Connection closed");
                        tx.send(GameMessage::ClientDisconnect(id)).await.unwrap();
                        return;
                    }
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
        let player = lock.truco.create_player(&params.name);
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
        lock.truco.create_room(&params.name);
    }
}

async fn rooms_handler(state: State<AppState>) -> impl IntoResponse {
    if let Ok(lock) = state.0.lock() {
        let rooms = lock.truco.get_rooms();
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
        let player = lock.truco.add_user_to_room(&params.room, params.user_id);
        Json(player).into_response()
    } else {
        Json("").into_response()
    }
}
