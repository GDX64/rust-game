use futures::{
    channel::mpsc::{Receiver, Sender},
    future::join_all,
    SinkExt, StreamExt,
};
use game_state::{
    BotPlayer, ChannelConstructor, GlExec, OnlineClient, OnlineClientChannel, RunningMode,
};
use std::{
    env,
    time::{Duration, SystemTime},
};
use tokio::{net::TcpStream, time::interval};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let exec = GlExec::new_global(std::time::SystemTime::now());
    tokio::spawn(async move {
        loop {
            exec.tick(SystemTime::now());
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    });

    let mut tasks = vec![];

    for _ in 0..10 {
        tasks.push(tokio::spawn(async move {
            loop {
                make_bot().await;
            }
        }));
    }

    join_all(tasks).await;
}

async fn make_bot() {
    let addr = env::var("SERVER_ADDR").unwrap_or("127.0.0.1:5000".into());
    let constructor = MyChannelConstructor { url: addr };
    let online_mode = OnlineClient::new(Box::new(constructor));
    let mut runner = RunningMode::new(Box::new(online_mode));
    let mut interval = interval(Duration::from_millis(16));
    let mut bot = BotPlayer::default();
    loop {
        let tick = interval.tick();
        tick.await;
        if bot.player.id != runner.id() {
            bot = BotPlayer::new(runner.id());
        }
        let dt = 0.016;
        runner.tick(dt);
        bot.tick(dt, runner.server_state());
        bot.player.collect_messages().into_iter().for_each(|msg| {
            runner.send_game_message(game_state::GameMessage::InputMessage(msg));
        });
        if bot.is_dead() {
            println!("bot is dead");
            break;
        }
    }
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
    let stream = TcpStream::connect(addr).await.expect("Failed to connect");
    let addr = format!("ws://{}/ws?server_id=AWS+SP1", addr);
    println!("Connecting to {addr}");
    let (ws_stream, _) = tokio_tungstenite::client_async(addr, stream)
        .await
        .map_err(|e| {
            println!("Error connecting to server: {}", e);
            e
        })
        .unwrap();

    return ws_stream;
}
