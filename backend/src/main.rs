use std::sync::Arc;

use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use canvas_game::BackendServer;
use futures_util::StreamExt;
use tokio::sync::{Mutex, MutexGuard};
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

    async fn get_game_server(&self) -> MutexGuard<BackendServer> {
        self.game_server.lock().await
    }
}

type AppState = Apps;
const TICK: f64 = 0.1;

fn init_logger() {
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}

#[tokio::main]
async fn main() {
    init_logger();
    let state: AppState = Apps::new();
    // build our application with a single route
    let backend_app = Router::new()
        .route("/", get(|| async { "Sanity Check" }))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("./dist"))
        .with_state(state.clone());
    let local_set = tokio::task::LocalSet::new();
    let tick_task = local_set.run_until(async {
        tokio::task::spawn_local(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs_f64(TICK));
            loop {
                interval.tick().await;
                state.get_game_server().await.tick(TICK).await;
            }
        })
        .await
        .unwrap();
    });

    let listener_game = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();

    let game_axum = axum::serve(listener_game, backend_app);
    let (_r1, _r2) = tokio::join!(async { game_axum.await }, async { tick_task.await });
}

async fn ws_handler(ws: WebSocketUpgrade, state: State<AppState>) -> impl IntoResponse {
    let res = ws.on_upgrade(move |ws| {
        println!("new ws connection received");
        return async move {
            let (send, mut receive) = ws.split();
            let id = { state.get_game_server().await.add_player(send) };
            loop {
                let msg = receive.next().await;
                match msg {
                    Some(Ok(Message::Binary(msg))) => {
                        println!("player sent message: {:?}", msg);
                        state.get_game_server().await.on_string_message(msg);
                    }
                    _ => {
                        state.get_game_server().await.disconnect_player(id);
                        return;
                    }
                }
            }
        };
    });
    res
}
