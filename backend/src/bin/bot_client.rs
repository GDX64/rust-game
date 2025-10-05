use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use game::GameDatabase;
use game_state::{
    ChannelConstructor, GameTrace, GlExec, OnlineClient, OnlineClientChannel, RunningEvent,
    RunningMode, RunningModeMessage, WrappedActor,
};
use std::{
    collections::HashMap,
    env,
    time::{Duration, Instant, SystemTime},
};
use tokio::{net::TcpStream, time::interval};
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
    let exec = GlExec::new_global(std::time::SystemTime::now());
    tokio::spawn(async move {
        loop {
            exec.tick(SystemTime::now());
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    });

    let (sender, fut) = GameDatabase::actor();
    tokio::spawn(fut);
    GameTrace::init_sender(sender);

    let t1 = tokio::spawn(make_bot_pool(10, false));
    let t2 = tokio::spawn(make_bot_pool(1, true));
    let _ = tokio::join!(t1, t2);
}

fn create_runner() -> WrappedActor<RunningMode> {
    let addr = env::var("SERVER_ADDR").unwrap_or("127.0.0.1:5000".into());
    let constructor = MyChannelConstructor { url: addr };
    let online_mode = OnlineClient::new(Box::new(constructor));
    let runner = RunningMode::wrapped(Box::new(online_mode));
    return runner;
}

async fn make_bot_pool(bots: usize, collect_stats: bool) {
    let mut runner = create_runner();
    let t1 = runner.listener(async |mut runner| -> () {
        let mut interval = interval(Duration::from_millis(16));
        loop {
            let tick = interval.tick();
            tick.await;
            let dt = 0.016;
            runner.send(RunningModeMessage::Tick(dt)).await;
        }
    });
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
                runner.send(RunningModeMessage::CreateBot).await;
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
        let mut interval = interval(Duration::from_secs(1));
        for _ in 0..bots {
            runner.send(RunningModeMessage::CreateBot).await;
            interval.tick().await;
        }
    });

    futures::join!(t1, t2, t3, t4);
}

struct MyChannelConstructor {
    url: String,
}

impl ChannelConstructor for MyChannelConstructor {
    fn new(&self) -> Box<dyn OnlineClientChannel> {
        Box::new(MyChannel::new(self.url.clone()))
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
    fn new(url: String) -> Self {
        let (w_sender, mut w_receiver) = futures::channel::mpsc::channel::<Vec<u8>>(10_000);
        let (mut r_sender, r_receiver) = futures::channel::mpsc::channel::<Vec<u8>>(10_000);
        tokio::spawn(async move {
            let ws = make_client(&url).await;
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

pub async fn make_client(addr: &str) -> tokio_tungstenite::WebSocketStream<TcpStream> {
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
    let addr = format!("ws://{}/ws?server_id=AWS+SP1", addr);
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
