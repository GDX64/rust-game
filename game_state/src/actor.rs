use std::{future::Future, sync::Arc};

use async_task::Task;
use futures::{
    channel::{
        mpsc::{channel, Sender},
        oneshot,
    },
    SinkExt, StreamExt,
};

use crate::GlExec;

pub trait GlActor: Send + 'static {
    type Message: Send + 'static;
    fn on_message(&mut self, msg: Self::Message) -> impl Future<Output = ()> + Send + '_;

    fn to_wrapped(self) -> WrappedActor<Self>
    where
        Self: Sized,
    {
        WrappedActor::new(self)
    }
}

enum WrappedMessage<A: GlActor> {
    Message(A::Message),
    WithState(Box<dyn FnOnce(&mut A) + Send>),
}

pub struct WrappedActor<A: GlActor> {
    sender: Sender<WrappedMessage<A>>,
    #[allow(dead_code)]
    task: Arc<Task<()>>,
}

impl<A: GlActor> Clone for WrappedActor<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            task: self.task.clone(),
        }
    }
}

impl<A: GlActor> WrappedActor<A> {
    pub fn new(actor: A) -> Self {
        let (sender, mut receiver) = channel(100);
        let task = GlExec::spawn(async move {
            let mut actor = actor;
            while let Some(msg) = receiver.next().await {
                match msg {
                    WrappedMessage::Message(msg) => {
                        actor.on_message(msg).await;
                    }
                    WrappedMessage::WithState(f) => {
                        f(&mut actor);
                    }
                }
            }
        });
        let wrapped = Self {
            sender,
            task: task.into(),
        };
        return wrapped;
    }

    pub async fn send(&mut self, msg: A::Message) {
        let msg = WrappedMessage::Message(msg);
        self.sender.send(msg).await.unwrap();
    }

    pub async fn with_state<T: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut A) -> T + Send + 'static,
    ) -> T {
        let (sender, receiver) = oneshot::channel();
        let f: Box<dyn FnOnce(&mut A) + Send> = Box::new(move |actor: &mut A| {
            let result = f(actor);
            match sender.send(result) {
                Ok(_) => {}
                Err(_) => {
                    eprintln!("Failed to send result");
                }
            }
        });
        let msg = WrappedMessage::WithState(f);
        self.sender.send(msg).await.unwrap();
        return receiver.await.unwrap();
    }
}

#[cfg(test)]
mod test {
    use crate::{GlActor, GlExec};

    #[test]
    fn interface_test() {
        #[derive(Debug)]
        enum Message {
            Inc(u32),
        }

        struct TestActor {
            counter: u32,
        }

        impl GlActor for TestActor {
            type Message = Message;
            async fn on_message(&mut self, msg: Self::Message) -> () {
                match msg {
                    Message::Inc(v) => self.counter += v,
                }
            }
        }

        let exec = GlExec::new_global(std::time::SystemTime::now());
        let test_actor = TestActor { counter: 0 };
        let mut wrapped = test_actor.to_wrapped();
        exec.block_on(async move {
            wrapped.send(Message::Inc(5)).await;
            wrapped.send(Message::Inc(5)).await;
            let counter = wrapped
                .with_state(|actor| {
                    return actor.counter;
                })
                .await;
            assert_eq!(counter, 10);
        })
    }
}
