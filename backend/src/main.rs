use std::{
    any::Any,
    sync::{Arc, Mutex, MutexGuard},
};

use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use canvas_game::BackendServer;
use futures_util::{SinkExt, StreamExt};
use game_state::{GameMessage, GameServer};
use tower_http::services::ServeDir;
mod canvas_game;

#[derive(Clone)]
struct Apps {
    game_server: Arc<Mutex<BackendServer>>,
}

impl Apps {
    fn new() -> Apps {
        Apps {
            game_server: Arc::new(Mutex::new(BackendServer::new())),
        }
    }

    fn get_game_server(&self) -> MutexGuard<BackendServer> {
        self.game_server.lock().unwrap()
    }
}

type AppState = Apps;

#[tokio::main]
async fn main() {
    let state: AppState = Apps::new();
    // build our application with a single route
    let backend_app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    let static_app = Router::new().nest_service("/", ServeDir::new("./dist"));

    let tick_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs_f64(0.016));
        loop {
            interval.tick().await;
            state.get_game_server().tick();
        }
    });

    let listener_game = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    let listener_static = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let static_axum = axum::serve(listener_static, static_app);
    let game_axum = axum::serve(listener_game, backend_app);
    let (_r1, _r2, _r3) = tokio::join!(
        async { game_axum.await },
        async { static_axum.await },
        async { tick_task.await }
    );
}

async fn ws_handler(ws: WebSocketUpgrade, state: State<AppState>) -> impl IntoResponse {
    let res = ws.on_upgrade(move |ws| {
        return async move {
            let (send, mut receive) = ws.split();
            let id = { state.get_game_server().add_player(send) };
            loop {
                let msg = receive.next().await;
                match msg {
                    Some(Ok(Message::Text(msg))) => {
                        state.get_game_server().on_string_message(msg);
                    }
                    _ => {
                        state.get_game_server().disconnect_player(id);
                        return;
                    }
                }
            }
        };
    });
    res
}
