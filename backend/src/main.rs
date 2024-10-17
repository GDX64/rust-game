use axum::{
    extract::{ws::Message, Query, State, WebSocketUpgrade},
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use database::GameDatabase;
use futures::{
    channel::mpsc::{channel, Sender},
    SinkExt,
};
use futures_util::StreamExt;
use game_state::{DBStatsMessage, TICK_TIME};
use server_pool::ServerPool;
use std::sync::{Arc, Mutex, MutexGuard};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
mod database;
mod server_pool;

const DB_PATH: &str = "./data/game.db";

#[derive(Clone)]
struct Apps {
    game_server: Arc<Mutex<ServerPool>>,
    stats_db: Arc<Mutex<GameDatabase>>,
}

impl Apps {
    fn new(db_sender: Sender<DBStatsMessage>) -> Apps {
        let mut pool = ServerPool::new(db_sender);
        pool.create_server("AWS SP1", 5)
            .expect("Failed to create default server");
        pool.create_server("AWS SP2", 1)
            .expect("Failed to create default server");

        let stats_db = GameDatabase::file(DB_PATH).expect("Failed to create db");

        Apps {
            game_server: Arc::new(Mutex::new(pool)),
            stats_db: Arc::new(Mutex::new(stats_db)),
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

    let static_dir = ServeDir::new("./dist");
    let static_dir = static_dir.fallback(ServeFile::new("./dist/index.html"));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    let (sender, future) = GameDatabase::actor(DB_PATH);

    let db_join = tokio::spawn(future);

    let state: AppState = Apps::new(sender);
    let backend_app = Router::new()
        .nest_service("/", static_dir)
        .route("/hello", get(|| async { "Sanity Check" }))
        .route("/ws", get(ws_handler))
        .route("/create_server", get(create_server_handler))
        .route("/get_server_list", get(get_server_list_handler))
        .route("/remove_server", get(remove_server_handler))
        .route("/get_player_id", get(handle_get_player_id))
        .route("/ranking", get(handle_ranking_stats))
        .route("/error", post(handle_post_error))
        .layer(CompressionLayer::new().gzip(true))
        .layer(cors)
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
    let (_r1, _r2, _r3) = tokio::join!(
        async { game_axum.await },
        async { tick_task.await },
        db_join
    );
}

#[derive(serde::Deserialize)]
struct WsQuery {
    server_id: String,
    player_name: Option<String>,
    player_id: Option<u64>,
    flag: Option<String>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    params: Query<WsQuery>,
    state: State<AppState>,
) -> impl IntoResponse {
    let server_id = params.server_id.clone();
    let player_name = params.player_name.clone().unwrap_or("Unknown".to_string());
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

async fn handle_get_player_id(params: Query<WsQuery>, state: State<AppState>) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    enum IdResponse {
        Ok(u64),
        Err(String),
    }

    let server_id = params.server_id.clone();
    log::info!("Getting player id to server {server_id}");
    let res = {
        if let Some(id) = state.get_game_server().get_player_id_for_server(&server_id) {
            log::info!("Id retrieved: {id}");
            IdResponse::Ok(id)
        } else {
            log::warn!("Server {server_id} not found");
            IdResponse::Err("Server not found".to_string())
        }
    };

    axum::Json(res)
}

async fn handle_ranking_stats(state: State<AppState>) -> impl IntoResponse {
    match state.stats_db.lock() {
        Ok(db) => {
            match db.get_leaderboard(15) {
                Ok(players) => {
                    return Ok(axum::Json(players));
                }
                Err(e) => {
                    log::error!("Failed to get leaderboard: {e}");
                }
            }
        }
        Err(e) => {
            log::error!("Failed to lock db: {e}");
        }
    };
    return Err(StatusCode::INTERNAL_SERVER_ERROR);
}

async fn handle_post_error(body: Json<serde_json::Value>) -> impl IntoResponse {
    let body_data = body.to_string();
    log::error!("Error received from client: {body_data}");
}

#[derive(serde::Deserialize)]
struct CreateServerParams {
    server_id: String,
    server_seed: u32,
}

async fn create_server_handler(
    params: Query<CreateServerParams>,
    state: State<AppState>,
) -> impl IntoResponse {
    match state
        .get_game_server()
        .create_server(&params.server_id, params.server_seed)
    {
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
