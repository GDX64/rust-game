use futures::{
    channel::mpsc::{channel, Receiver},
    StreamExt,
};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{MessageEvent, WebSocket};

pub struct WSChannel {
    receiver: Receiver<Vec<u8>>,
    ws: WebSocket,
}

impl WSChannel {
    pub fn new(url: &str) -> Self {
        let (mut channel_sender, channel_receiver) = channel(1000);
        let ws = WebSocket::new(url).expect("Failed to create WebSocket");
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let on_message = Closure::new(move |event: MessageEvent| {
            let data = event.data().dyn_into::<js_sys::ArrayBuffer>();
            if let Ok(data) = data {
                let data = js_sys::Uint8Array::new(&data).to_vec();
                channel_sender
                    .try_send(data)
                    .expect("Failed to send message on websocket internal channel");
            }
        });
        let on_message = on_message.into_js_value();
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        WSChannel {
            receiver: channel_receiver,
            ws,
        }
    }

    pub fn send(&mut self, msg: Vec<u8>) {
        let msg = js_sys::Uint8Array::from(msg.as_slice());
        let msg = msg.buffer();
        self.ws
            .send_with_array_buffer(&msg)
            .expect("Failed to send message on websocket");
    }

    pub fn receive(&mut self) -> Option<Vec<u8>> {
        match self.receiver.try_next() {
            Ok(Some(msg)) => Some(msg),
            _ => None,
        }
    }

    pub async fn next(&mut self) -> Option<Vec<u8>> {
        self.receiver.next().await
    }
}
