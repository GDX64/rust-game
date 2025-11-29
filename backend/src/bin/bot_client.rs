use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use game::GameDatabase;
use game_state::{
    ChannelConstructor, GameTrace, OnlineClient, OnlineClientChannel, RunningEvent, RunningMode,
    WrappedActor,
};
use std::{
    collections::HashMap,
    env,
    time::{Duration, Instant},
};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

fn init_logger() {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() {
    init_logger();
    let (sender, fut) = GameDatabase::actor();
    tokio::spawn(fut);
    GameTrace::init_sender(sender);

    let server_count: usize = env::var("INITIAL_SERVER_COUNT").unwrap().parse().unwrap();
    let mut handlers = vec![];
    for i in 0..server_count {
        let handler = tokio::spawn(async move {
            let bot_count: usize = env::var("BOT_COUNT")
                .unwrap_or("2".to_string())
                .parse()
                .unwrap();
            make_bot_pool(bot_count, false, i).await;
        });
        handlers.push(handler);
    }
    for handler in handlers {
        let _ = handler.await;
    }
}

fn create_runner(server_num: usize) -> WrappedActor<RunningMode> {
    let addr = env::var("SERVER_ADDR").unwrap();
    let constructor = MyChannelConstructor {
        url: addr,
        server_num,
    };
    let online_mode = OnlineClient::new(Box::new(constructor));
    let runner = RunningMode::wrapped(Box::new(online_mode));
    return runner;
}

async fn make_bot_pool(bots: usize, collect_stats: bool, server_num: usize) {
    let mut runner = create_runner(server_num);
    let mut sub = runner.subscribe().await;
    while let Some(msg) = sub.receiver.next().await {
        if let RunningEvent::Connected = msg {
            break;
        }
    }

    let t2 = runner.listener(async |mut runner| {
        let mut sub = runner.subscribe().await;
        while let Some(event) = sub.receiver.next().await {
            if let RunningEvent::BotDead(_) = event {
                runner.with_state(|s| s.ask_create_player(true)).await;
            }
        }
    });

    let t3 = runner.listener(async move |mut runner| {
        if !collect_stats {
            return;
        }
        let mut sub = runner.subscribe().await;
        struct PingData {
            start: Instant,
            tick: u64,
        }
        let mut ping_map = HashMap::<u64, PingData>::new();
        let mut player_id = runner.with_state(|s| s.id()).await;
        while let Some(event) = sub.receiver.next().await {
            match event {
                RunningEvent::PlayerCreated { id, .. } => {
                    player_id = id;
                }
                RunningEvent::StatePong { id } => {
                    if let Some(data) = ping_map.remove(&id) {
                        let elapsed = data.start.elapsed();
                        GameTrace::PingTime {
                            micros: elapsed.as_micros() as u64,
                            player_id,
                            tick: data.tick,
                        }
                        .send();
                    }
                }
                RunningEvent::Tick { tick } => {
                    let id = runner.with_state(|s| s.ask_ping()).await;
                    ping_map.insert(
                        id,
                        PingData {
                            start: Instant::now(),
                            tick,
                        },
                    );
                }
                _ => {}
            }
        }
    });

    let t4 = runner.listener(async move |mut runner| {
        if collect_stats {
            return;
        }
        for _ in 0..bots {
            runner.with_state(|s| s.ask_create_player(true)).await;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    let _ = tokio::join!(t2, t3, t4);
}

struct MyChannelConstructor {
    url: String,
    server_num: usize,
}

impl ChannelConstructor for MyChannelConstructor {
    fn new(&self) -> Box<dyn OnlineClientChannel> {
        Box::new(MyChannel::new(self.url.clone(), self.server_num))
    }
}

struct MyChannel {
    receiver: Option<Receiver<Vec<u8>>>,
    sender: Sender<Vec<u8>>,
}

impl Drop for MyChannel {
    fn drop(&mut self) {
        self.sender.close_channel();
        println!("Channel dropped");
    }
}

impl MyChannel {
    fn new(url: String, server_num: usize) -> Self {
        let (w_sender, mut w_receiver) = futures::channel::mpsc::channel::<Vec<u8>>(10_000);
        let (mut r_sender, r_receiver) = futures::channel::mpsc::channel::<Vec<u8>>(10_000);
        tokio::spawn(async move {
            let ws = make_client(&url, server_num).await;
            let (mut write, mut read) = ws.split();
            let f1 = async move {
                while let Some(msg) = w_receiver.next().await {
                    if let Err(e) = write.send(Message::Binary(msg.into())).await {
                        eprintln!("Error sending message: {}", e);
                        break;
                    }
                }
                write.close().await.ok();
                w_receiver.close();
                println!("WebSocket write loop ended")
            };
            let f2 = async move {
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Binary(msg)) => {
                            if let Err(e) = r_sender.try_send(msg.into()) {
                                eprintln!("Error sending to channel: {}", e);
                                break;
                            }
                        }
                        _ => {
                            eprintln!("Error receiving message");
                            break;
                        }
                    }
                }
                r_sender.close().await.ok();
                println!("WebSocket read loop ended");
            };
            return futures::join!(f1, f2);
        });
        Self {
            receiver: Some(r_receiver),
            sender: w_sender,
        }
    }
}

impl OnlineClientChannel for MyChannel {
    fn send(&mut self, msg: Vec<u8>) {
        self.sender.try_send(msg).unwrap();
    }

    fn receiver(&mut self) -> Option<Receiver<Vec<u8>>> {
        return self.receiver.take();
    }
}

pub async fn make_client(
    addr: &str,
    server_num: usize,
) -> tokio_tungstenite::WebSocketStream<TcpStream> {
    log::info!("Connecting to server at {}", addr);
    let stream = loop {
        let stream = TcpStream::connect(addr).await;
        match stream {
            Ok(s) => break s,
            Err(e) => {
                log::error!("Error connecting to server: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    let addr = format!("ws://{}/ws?server_id=AWS+SP{}", addr, server_num);
    log::info!("Connecting to WebSocket at {}", addr);
    let (ws_stream, _) = tokio_tungstenite::client_async(addr, stream)
        .await
        .map_err(|e| {
            log::error!("Error connecting to WebSocket: {}", e);
            e
        })
        .unwrap();

    return ws_stream;
}
