use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use game_state::{ChannelConstructor, OnlineClient, OnlineClientChannel, RunningMode};
use std::{env, time::Duration};
use tokio::{net::TcpStream, time::interval};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() {
    let addr = env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    let constructor = MyChannelConstructor { url: addr };
    let online_mode = OnlineClient::new(Box::new(constructor));
    let mut runner = RunningMode::new(Box::new(online_mode));
    let mut interval = interval(Duration::from_millis(16));
    loop {
        interval.tick().await;
        runner.tick(0.016);
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
            while let Some(msg) = w_receiver.next().await {
                write.send(Message::Binary(msg.into())).await.unwrap();
            }
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
    let (ws_stream, _) = tokio_tungstenite::client_async(format!("ws://{}", addr), stream)
        .await
        .expect("Failed to connect");

    return ws_stream;
}
