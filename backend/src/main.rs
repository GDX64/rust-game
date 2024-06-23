use std::sync::{Arc, Mutex};

use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use game_state::GameMessage;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;
mod canvas_game;

struct Apps {
    canvas_tx: tokio::sync::mpsc::Sender<GameMessage>,
}

impl Apps {
    fn new(tx: tokio::sync::mpsc::Sender<GameMessage>) -> Apps {
        Apps { canvas_tx: tx }
    }
}

type AppState = Arc<Mutex<Apps>>;

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    tokio::spawn(canvas_game::run(rx));
    let state: AppState = Arc::new(Mutex::new(Apps::new(tx)));
    // build our application with a single route
    let backend_app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", get(ws_handler))
        .with_state(state);

    let static_app = Router::new().nest_service("/", ServeDir::new("./dist"));

    let listener_game = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    let listener_static = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let static_axum = axum::serve(listener_static, static_app);
    let game_axum = axum::serve(listener_game, backend_app);
    let (_r1, _r2) = tokio::join!(async { game_axum.await }, async { static_axum.await });
}

async fn ws_handler(ws: WebSocketUpgrade, state: State<AppState>) -> impl IntoResponse {
    let tx = state.lock().unwrap().canvas_tx.clone();
    let res = ws.on_upgrade(move |ws| {
        async move {
            let (mut send, mut receive) = ws.split();
            let (send_id, receive_id) = oneshot::channel();
            let (client_tx, mut client_rx) = tokio::sync::mpsc::channel(10);
            let handler = tokio::spawn(async move {
                loop {
                    let msg = client_rx.recv().await;
                    match msg {
                        Some(msg) => {
                            if let Err(err) = send.send(Message::Text(msg)).await {
                                eprintln!("Error sending message: {:?}", err)
                            }
                        }
                        None => {
                            return;
                        }
                    }
                }
            });

            let send_result = tx
                .send(GameMessage::NewConnection {
                    id_sender: send_id,
                    sender: client_tx,
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
                        handler.abort();
                        return;
                    }
                }
            }
        }
    });
    res
}
