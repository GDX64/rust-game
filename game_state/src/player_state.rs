use serde::{Deserialize, Serialize};

use crate::get_flag_names;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PlayerState {
    pub name: String,
    pub id: u64,
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
            id: 0,
            percentage_of_map: 0.0,
            islands: 0,
            ships: 0,
            kills: 0,
            flag: get_player_flag(0),
            deaths: 0,
        }
    }
}

fn get_player_flag(id: u64) -> String {
    let flags = get_flag_names();
    let index = fastrand::Rng::with_seed(id).usize(0..flags.len());
    get_flag_names()[index].into()
}

impl PlayerState {
    pub fn update_flag(&mut self) {
        self.flag = get_player_flag(self.id);
    }

    pub fn new(name: String, id: u64) -> Self {
        PlayerState {
            name,
            id,
            flag: get_player_flag(id),
            ..Default::default()
        }
    }
}
