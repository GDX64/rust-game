use std::future::Future;

use futures::FutureExt;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use web_sys::window;

pub struct WasmSleep {
    receiver: futures::channel::oneshot::Receiver<()>,
    timeout: i32,
}

impl WasmSleep {
    pub async fn sleep(time: i32) {
        let f = wasm_bindgen_futures::JsFuture::from(sleep(time)).await;
        match f {
            Ok(_) => (),
            Err(e) => log::error!("Failed to sleep: {:?}", e),
        }
    }

    fn new(time: i32) -> Self {
        let (sender, receiver) = futures::channel::oneshot::channel();

        let f = Closure::once(move || {
            match sender.send(()) {
                Ok(_) => (),
                Err(e) => log::error!("Failed to send message: {:?}", e),
            }
        });
        let f = f.into_js_value();

        let id = window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(f.unchecked_ref(), time);

        WasmSleep {
            receiver,
            timeout: id.unwrap_or_default(),
        }
    }
}

impl Future for WasmSleep {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.receiver.poll_unpin(cx) {
            std::task::Poll::Ready(_) => std::task::Poll::Ready(()),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
        // std::task::Poll::Pending
    }
}

impl Drop for WasmSleep {
    fn drop(&mut self) {
        window().unwrap().clear_timeout_with_handle(self.timeout);
    }
}

#[wasm_bindgen(module = "/src/js/scheduler.js")]
extern "C" {
    fn sleep(time: i32) -> Promise;
}
