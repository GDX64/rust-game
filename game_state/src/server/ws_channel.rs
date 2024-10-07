use futures::{
    channel::mpsc::{channel, Receiver},
    StreamExt,
};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{MessageEvent, WebSocket};

pub struct WSChannel {
    receiver: Option<Receiver<Vec<u8>>>,
    ws: WebSocket,
}

impl WSChannel {
    pub fn new(url: &str) -> Self {
        let (mut channel_sender, channel_receiver) = channel(1000);
        let ws = WebSocket::new(url).expect("Failed to create WebSocket");
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let mut on_message_sender = channel_sender.clone();
        let on_message = Closure::new(move |event: MessageEvent| {
            let data = event.data().dyn_into::<js_sys::ArrayBuffer>();
            if let Ok(data) = data {
                let data = js_sys::Uint8Array::new(&data).to_vec();
                match on_message_sender.try_send(data) {
                    Ok(_) => (),
                    Err(e) => {
                        log::error!("Failed to send message: {:?}", e)
                    }
                }
            }
        });

        let on_close = Closure::new(move |_event: MessageEvent| {
            channel_sender.close_channel();
            log::info!("WebSocket closed");
        });

        let on_close = on_close.into_js_value();
        let on_message = on_message.into_js_value();
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        WSChannel {
            receiver: Some(channel_receiver),
            ws,
        }
    }

    pub fn send(&mut self, msg: Vec<u8>) -> Option<()> {
        let msg = js_sys::Uint8Array::from(msg.as_slice());
        let msg = msg.buffer();
        self.ws.send_with_array_buffer(&msg).ok()
    }

    pub fn is_offline(&self) -> bool {
        self.ws.ready_state() == WebSocket::CLOSED
    }

    pub fn is_connecting(&self) -> bool {
        self.ws.ready_state() == WebSocket::CONNECTING
    }

    pub fn receiver(&mut self) -> Option<Receiver<Vec<u8>>> {
        self.receiver.take()
    }

    pub async fn next(&mut self) -> Option<Vec<u8>> {
        self.receiver.as_mut()?.next().await
    }
}

impl Drop for WSChannel {
    fn drop(&mut self) {
        match self.ws.close() {
            Ok(_) => (),
            Err(e) => log::error!("Failed to close websocket: {:?}", e),
        }
    }
}
