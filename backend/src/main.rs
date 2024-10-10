use axum::{
    extract::{ws::Message, Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{channel::mpsc::channel, SinkExt};
use futures_util::StreamExt;
use game_state::TICK_TIME;
use server_pool::ServerPool;
use std::sync::{Arc, Mutex, MutexGuard};
use tower_http::{compression::CompressionLayer, services::ServeDir};
mod server_pool;

#[derive(Clone)]
struct Apps {
    game_server: Arc<Mutex<ServerPool>>,
}

impl Apps {
    fn new() -> Apps {
        let mut pool = ServerPool::new();
        pool.create_server("default")
            .expect("Failed to create default server");

        Apps {
            game_server: Arc::new(Mutex::new(pool)),
        }
    }

    fn get_game_server(&self) -> MutexGuard<ServerPool> {
        self.game_server.lock().expect("Failed to lock game server")
    }
}

type AppState = Apps;

fn init_logger() {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() {
    init_logger();
    log::info!("Starting Axum Server");

    let state: AppState = Apps::new();
    // build our application with a single route
    let backend_app = Router::new()
        .route("/hello", get(|| async { "Sanity Check" }))
        .route("/ws", get(ws_handler))
        .route("/create_server", get(create_server_handler))
        .route("/get_server_list", get(get_server_list_handler))
        .route("/remove_server", get(remove_server_handler))
        .nest_service("/", ServeDir::new("./dist"))
        .layer(CompressionLayer::new().gzip(true))
        .with_state(state.clone());

    let local_set = tokio::task::LocalSet::new();
    let tick_task = local_set.run_until(async {
        tokio::task::spawn_local(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs_f64(TICK_TIME));
            loop {
                interval.tick().await;
                state.get_game_server().tick(TICK_TIME);
            }
        })
        .await
        .unwrap();
    });

    let listener_game = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();

    let game_axum = axum::serve(listener_game, backend_app);
    let (_r1, _r2) = tokio::join!(async { game_axum.await }, async { tick_task.await });
}

#[derive(serde::Deserialize)]
struct WsQuery {
    server_id: String,
    player_name: String,
    player_id: Option<u64>,
    flag: Option<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    params: Query<WsQuery>,
    state: State<AppState>,
) -> impl IntoResponse {
    let server_id = params.server_id.clone();
    let player_name = params.player_name.clone();
    let player_id = params.player_id.clone();
    let flag = params.flag.clone();
    log::info!("Connecting {player_name} Player to server {server_id}");
    let res = ws.on_upgrade(move |ws| {
        return async move {
            let (mut send, mut receive) = ws.split();
            let (player_send, mut player_receive) = channel(100);

            tokio::spawn(async move {
                while let Some(msg) = player_receive.next().await {
                    match send.send(Message::Binary(msg)).await {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    }
                }
            });
            let id = {
                if let Some(server) = state.get_game_server().get_server(&server_id) {
                    server.new_connection(player_send, player_id, &player_name, flag)
                } else {
                    log::warn!("Server {server_id} not found, disconnecting player {player_name}");
                    return;
                }
            };
            log::info!("Player {player_name} connected to server {server_id} with id {id}");
            loop {
                let msg = receive.next().await;
                match msg {
                    Some(Ok(Message::Binary(msg))) => {
                        match state.get_game_server().get_server(&server_id) {
                            Some(server) => {
                                server.on_message(msg);
                            }
                            None => {
                                log::warn!("Server {server_id} not found, disconnecting player");
                                return;
                            }
                        }
                    }
                    _ => {
                        log::info!("Player {id} disconnected");
                        state
                            .get_game_server()
                            .get_server(&server_id)
                            .map(|server| {
                                server.on_player_connection_down(id);
                            });
                        return;
                    }
                }
            }
        };
    });
    res
}

#[derive(serde::Deserialize)]
struct CreateServerParams {
    server_id: String,
}

async fn create_server_handler(
    params: Query<CreateServerParams>,
    state: State<AppState>,
) -> impl IntoResponse {
    match state.get_game_server().create_server(&params.server_id) {
        Ok(_) => {
            let server_id = params.server_id.clone();
            log::info!("Server {server_id} created");
            return "Server created".to_string();
        }
        Err(e) => {
            log::error!("Failed to create server: {e}");
            return format!("Failed to create server: {}", e);
        }
    }
}

async fn get_server_list_handler(state: State<AppState>) -> impl IntoResponse {
    let servers = state.get_game_server().get_server_info();
    return axum::Json(servers);
}

async fn remove_server_handler(
    params: Query<CreateServerParams>,
    state: State<AppState>,
) -> impl IntoResponse {
    match state.get_game_server().remove_server(&params.server_id) {
        Ok(_) => {
            let server_id = params.server_id.clone();
            log::info!("Server {server_id} removed");
            return "Server removed".to_string();
        }
        Err(e) => {
            log::error!("Failed to remove server: {e}");
            return format!("Failed to remove server: {}", e);
        }
    }
}
