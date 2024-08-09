use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{channel::mpsc::channel, SinkExt};
use futures_util::StreamExt;
use game_state::GameServer;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct Apps {
    game_server: Arc<Mutex<GameServer>>,
}

impl Apps {
    fn new() -> Apps {
        Apps {
            game_server: Arc::new(Mutex::new(GameServer::new())),
        }
    }

    async fn get_game_server(&self) -> MutexGuard<GameServer> {
        self.game_server.lock().await
    }
}

type AppState = Apps;
const TICK: f64 = 1.0 / 60.0;

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
                state.get_game_server().await.tick(TICK);
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
            let (mut send, mut receive) = ws.split();
            let (player_send, mut player_receive) = channel(100);
            tokio::spawn(async move {
                while let Some(msg) = player_receive.next().await {
                    send.send(Message::Binary(msg)).await.unwrap();
                }
            });
            let id = { state.get_game_server().await.new_connection(player_send) };
            loop {
                let msg = receive.next().await;
                match msg {
                    Some(Ok(Message::Binary(msg))) => {
                        state.get_game_server().await.on_message(msg);
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
