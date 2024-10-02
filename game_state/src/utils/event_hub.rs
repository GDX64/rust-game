use futures::{
    channel::mpsc::{Receiver, Sender},
    StreamExt,
};
use js_sys::Promise;
use serde::Serialize;
use std::{future::Future, mem};
use wasm_bindgen::JsValue;

pub trait EventKey: Clone + PartialEq + 'static {}
pub struct EventHub<K: EventKey> {
    senders: Vec<Sender<K>>,
}

pub struct Subscription<T> {
    receiver: Receiver<T>,
}

impl<K: EventKey> EventHub<K> {
    pub fn new() -> Self {
        Self {
            senders: Vec::new(),
        }
    }

    pub fn subscribe(&mut self) -> Subscription<K> {
        let (sender, receiver) = futures::channel::mpsc::channel(10);
        self.senders.push(sender);
        Subscription { receiver }
    }

    pub fn notify(&mut self, event: K) {
        let senders = mem::take(&mut self.senders);
        self.senders = senders
            .into_iter()
            .filter_map(|mut sender| {
                match sender.try_send(event.clone()) {
                    Ok(_) => {
                        return Some(sender);
                    }
                    Err(_) => {
                        return None;
                    }
                }
            })
            .collect();
    }

    pub fn as_promise<T, F>(&mut self, f: F) -> Promise
    where
        T: Serialize + 'static,
        F: Fn(K) -> Option<T> + 'static,
    {
        let mut recv = self.subscribe();
        let future = async move {
            while let Some(event) = recv.receiver.next().await {
                if let Some(val) = f(event) {
                    return Ok(val);
                }
            }
            return Err(anyhow::anyhow!("No event received"));
        };

        return as_promise(future);
    }
}

fn as_promise<T: Serialize + 'static>(
    val: impl Future<Output = anyhow::Result<T>> + 'static,
) -> Promise {
    let mapped = async move {
        let res = val
            .await
            .and_then(|v| {
                return serde_wasm_bindgen::to_value(&v)
                    .map_err(|err| anyhow::anyhow!("Error serializing value: {:?}", err));
            })
            .map_err(|err| {
                let err: JsValue = js_sys::Error::new(&format!("{:?}", err)).into();
                return err;
            });
        return res;
    };
    return wasm_bindgen_futures::future_to_promise(mapped);
}
