use std::sync::Mutex;

use futures::channel::mpsc::Sender;

use crate::player_state::PlayerID;

pub enum GameTrace {
    PlayerConnected {
        player_id: PlayerID,
        game_id: u64,
        player_name: String,
    },
    PlayerDisconnected {
        player_id: PlayerID,
        game_id: u64,
    },
    ShipDestroyed {
        ship_id: u64,
        player_id: PlayerID,
        killed_by: PlayerID,
        frame: u64,
        game_id: u64,
    },
    ServerTick {
        game_id: u64,
        tick: u64,
        micros_elapsed: u64,
    },
    PingTime {
        micros: u64,
        player_id: PlayerID,
        tick: u64,
    },
}

static STATS_SENDER: Mutex<Option<Sender<GameTrace>>> = Mutex::new(None);

impl GameTrace {
    pub fn send(self) {
        if let Some(sender) = STATS_SENDER.lock().unwrap().as_mut() {
            sender.try_send(self).ok();
        }
    }

    pub fn init_sender(sender: Sender<GameTrace>) {
        let mut guard = STATS_SENDER.lock().unwrap();
        *guard = Some(sender);
    }
}
