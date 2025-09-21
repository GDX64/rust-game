use serde::{Deserialize, Serialize};

use crate::get_flag_names;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PlayerState {
    pub name: String,
    pub id: PlayerID,
    pub percentage_of_map: f64,
    pub islands: usize,
    pub ships: usize,
    pub kills: usize,
    pub deaths: usize,
    pub flag: String,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            name: "".to_string(),
            id: PlayerID::new(0),
            percentage_of_map: 0.0,
            islands: 0,
            ships: 0,
            kills: 0,
            flag: PlayerState::get_player_flag(PlayerID::new(0)),
            deaths: 0,
        }
    }
}

impl PlayerState {
    pub fn get_player_flag(id: PlayerID) -> String {
        let flags = get_flag_names();
        let index = fastrand::Rng::with_seed(id.as_u64()).usize(0..flags.len());
        get_flag_names()[index].into()
    }

    pub fn new(name: String, id: PlayerID, flag: String) -> Self {
        PlayerState {
            name,
            id,
            flag,
            ..Default::default()
        }
    }
}

#[derive(
    PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Debug, Default, PartialOrd, Ord,
)]
pub struct PlayerID(u64);

impl PlayerID {
    pub fn new(id: u64) -> PlayerID {
        PlayerID(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<PlayerID> for u64 {
    fn from(value: PlayerID) -> Self {
        value.0
    }
}
