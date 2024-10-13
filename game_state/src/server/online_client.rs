use crate::{
    server_state::ServerState,
    utils::scheduling::{Interval, WasmSleep},
};

use super::{game_server::GameMessage, local_client::Client, ws_channel::WSChannel};
use actor::Actor;
use futures::{join, select, stream::FusedStream, FutureExt, SinkExt, StreamExt};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct OnlineClient {
    actor: Option<Actor<GameMessage>>,
    url: String,
}

#[wasm_bindgen]
impl OnlineClient {
    pub fn new(url: &str) -> OnlineClient {
        let mut client = OnlineClient {
            actor: None,
            url: url.to_string(),
        };
        client.reconnect();
        client
    }

    fn next(&mut self) -> Option<GameMessage> {
        self.actor.as_mut()?.receiver.try_next().ok()?
    }
}

impl Client for OnlineClient {
    fn send(&mut self, msg: GameMessage) {
        if let Some(actor) = self.actor.as_mut() {
            match actor.sender.try_send(msg) {
                Ok(_) => (),
                Err(e) => log::error!("Failed to send message: {:?}", e),
            }
        }
    }

    fn next_message(&mut self) -> Option<GameMessage> {
        self.next()
    }

    fn tick(&mut self, _dt: f64) {
        //
    }

    fn server_state(&self) -> Option<&ServerState> {
        return None;
    }

    fn reconnect(&mut self) {
        let url = self.url.clone();

        let actor = Actor::<GameMessage>::spawn(move |mut sender, mut receiver| {
            let mut ws = WSChannel::new(&url);

            let mut ws_receiver = ws.receiver().expect("Failed to get receiver");

            let sender_future = async move {
                loop {
                    if receiver.is_terminated() {
                        break;
                    }
                    let borrowed = &mut receiver;
                    let v = borrowed.take_until(WasmSleep::sleep(16)).collect().await;
                    ws.send(GameMessage::serialize_arr(&v));
                }
            };

            let receiver_future = async move {
                log::info!("Reconnecting to {}", url);
                let mut interval = Interval::new(1000);
                let mut ticks_idle = 0;
                loop {
                    let ans = select! {
                        ans = ws_receiver.next().fuse() => {
                            ticks_idle = 0;
                            ans
                        },
                        _ = interval.tick().fuse() => {
                            ticks_idle += 1;
                            if ticks_idle > 5{
                                log::warn!("Connection idle detected");
                                None
                            }else{
                                continue;
                            }
                        }
                    };
                    match ans {
                        Some(msg) => {
                            let msg = GameMessage::from_arr_bytes(&msg);
                            msg.into_iter().for_each(|msg| {
                                match sender.try_send(msg) {
                                    Err(e) => log::error!("Failed to send message: {:?}", e),
                                    _ => (),
                                }
                            });
                        }
                        None => {
                            log::warn!("Connection down detected");
                            sender
                                .send(GameMessage::ConnectionDown)
                                .await
                                .expect("Failed to send");
                            break;
                        }
                    }
                }
            };
            return async move {
                join!(receiver_future, sender_future);
            };
        });
        self.actor = Some(actor);
    }
}

mod actor {
    use std::future::Future;

    use futures::channel::mpsc::{channel, Receiver, Sender};

    pub struct Actor<T> {
        pub sender: Sender<T>,
        pub receiver: Receiver<T>,
    }

    impl<T> Actor<T> {
        pub fn spawn<F: Future<Output = ()> + 'static>(
            f: impl FnOnce(Sender<T>, Receiver<T>) -> F,
        ) -> Actor<T> {
            let (sender_actor, receiver_main) = channel(100);
            let (sender_main, receiver_actor) = channel(100);
            let future = f(sender_actor, receiver_actor);
            wasm_bindgen_futures::spawn_local(future);
            return Actor {
                sender: sender_main,
                receiver: receiver_main,
            };
        }
    }
}
