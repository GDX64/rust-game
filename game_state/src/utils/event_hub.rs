use std::{any::Any, collections::HashMap, future::Future};

use futures::channel::oneshot;
use js_sys::Promise;
use serde::Serialize;
use wasm_bindgen::JsValue;

pub trait EventKey: Clone + Eq + std::hash::Hash {}
type BoxAny = Box<dyn Any>;
pub struct EventHub<K: EventKey> {
    event_map: HashMap<K, Vec<BoxAny>>,
}

impl<K: EventKey> EventHub<K> {
    pub fn new() -> Self {
        Self {
            event_map: HashMap::new(),
        }
    }

    pub fn when<T: 'static>(&mut self, event: K) -> impl Future<Output = anyhow::Result<T>> {
        let (sender, receiver) = oneshot::channel::<T>();
        let entry = self.event_map.entry(event);
        let v = entry.or_insert(vec![]);
        v.push(Box::new(sender));
        async move {
            let r = receiver.await;
            r.map_err(|_| anyhow::anyhow!("Channel was canceled"))
        }
    }

    pub fn notify<T: 'static + Clone>(&mut self, event: K, data: T) {
        self.event_map
            .remove(&event)
            .into_iter()
            .flatten()
            .for_each(|sender| {
                let sender = sender.downcast::<oneshot::Sender<T>>();
                if let Ok(sender) = sender {
                    sender.send(data.clone()).ok();
                }
            })
    }
}

#[cfg(target_arch = "wasm32")]
pub fn as_promise<T: Serialize + 'static>(
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
