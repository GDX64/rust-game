use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use game_state::{
    BotPlayer, ChannelConstructor, OnlineClient, OnlineClientChannel, RunningEvent, RunningMode,
};
use std::{env, time::Duration};
use tokio::{net::TcpStream, select, time::interval};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let addr = env::var("SERVER_ADDR").unwrap_or("127.0.0.1:5000".into());
    let constructor = MyChannelConstructor { url: addr };
    let online_mode = OnlineClient::new(Box::new(constructor));
    let mut runner = RunningMode::new(Box::new(online_mode));
    let mut interval = interval(Duration::from_millis(16));
    let mut bot = BotPlayer::new(0);
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

impl MyChannel {
    fn new(url: String) -> Self {
        let (w_sender, mut w_receiver) = futures::channel::mpsc::channel::<Vec<u8>>(100);
        let (mut r_sender, r_receiver) = futures::channel::mpsc::channel::<Vec<u8>>(100);
        tokio::spawn(async move {
            let ws = make_client(&url).await;
            let (mut write, mut read) = ws.split();
            let f1 = async move {
                while let Some(msg) = w_receiver.next().await {
                    println!("Sending a message to server");
                    write.send(Message::Binary(msg.into())).await.unwrap();
                }
            };
            let f2 = async move {
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Binary(msg)) => {
                            r_sender.try_send(msg.into()).unwrap();
                        }
                        _ => {
                            log::error!("Error receiving message");
                        }
                    }
                }
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
