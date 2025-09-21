use crate::{server::Client, GlExec};

use super::game_server::GameMessage;
use actor::Actor;
use async_task::Task;
use futures::{join, SinkExt, StreamExt};

pub trait ChannelConstructor: Send {
    fn new(&self) -> Box<dyn OnlineClientChannel>;
}

pub trait OnlineClientChannel: Send {
    fn send(&mut self, msg: Vec<u8>);
    fn receiver(&mut self) -> Option<futures::channel::mpsc::Receiver<Vec<u8>>>;
}

pub struct OnlineClient {
    actor: Option<Actor<GameMessage>>,
    constructor: Box<dyn ChannelConstructor>,
    task: Option<Task<()>>,
}

impl OnlineClient {
    pub fn new(constructor: Box<dyn ChannelConstructor>) -> OnlineClient {
        let mut client = OnlineClient {
            actor: None,
            constructor,
            task: None,
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

    fn tick(&mut self, _dt: f64) {}

    fn reconnect(&mut self) {
        let mut ws = self.constructor.new();

        let (actor, future) = Actor::<GameMessage>::spawn(move |mut sender, mut receiver| {
            let mut ws_receiver = ws.receiver().expect("Failed to get receiver");

            let sender_future = async move {
                loop {
                    if let Some(msg) = receiver.next().await {
                        let mut v = vec![msg];
                        while let Ok(Some(value)) = receiver.try_next() {
                            v.push(value);
                        }
                        ws.send(GameMessage::serialize_arr(&v));
                    } else {
                        break;
                    }
                }
            };

            let receiver_future = async move {
                log::info!("Reconnecting...");
                loop {
                    match ws_receiver.next().await {
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
        let task = GlExec::spawn(future);
        self.task = Some(task);
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
        pub fn spawn<F: Future<Output = ()> + 'static + Send>(
            f: impl FnOnce(Sender<T>, Receiver<T>) -> F,
        ) -> (Actor<T>, F) {
            let (sender_actor, receiver_main) = channel(10_000);
            let (sender_main, receiver_actor) = channel(10_000);
            let future = f(sender_actor, receiver_actor);
            return (
                Actor {
                    sender: sender_main,
                    receiver: receiver_main,
                },
                future,
            );
        }
    }
}
