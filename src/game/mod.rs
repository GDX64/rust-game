use std::collections::HashMap;

use serde::Serialize;

use self::game::{Game, GamePlayer, Player};

mod deck;
mod game;

enum AppAction {
    PlayCard(Player, String),
    CreatePlayer(String),
    CreateRoom,
}

enum RoomState {
    Waiting,
    Playing(Game),
    Finished,
}

#[derive(Clone, Serialize)]
struct GameState {
    players: Vec<GamePlayer>,
    turn: Option<Player>,
    team1_points: usize,
    team2_points: usize,
    team1_round_points: usize,
    team2_round_points: usize,
}

pub struct Room {
    pub players: Vec<Player>,
    state: RoomState,
    id: u64,
    name: String,
}

impl Room {
    pub fn new(name: String) -> Self {
        Self {
            players: Vec::new(),
            state: RoomState::Waiting,
            id: 0,
            name,
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.push(player);
    }

    pub fn remove_player(&mut self, player: Player) {
        self.players.retain(|p| p.id != player.id);
    }

    pub fn start_game(&mut self) {
        if let Ok(players) = self.players.clone().try_into() {
            self.state = RoomState::Playing(Game::new(players));
        }
    }
}

pub struct TrucoApp {
    rooms: HashMap<String, Room>,
    players: HashMap<u64, Player>,
    id: u64,
}

impl TrucoApp {
    pub fn new() -> Self {
        Self {
            id: 0,
            rooms: HashMap::new(),
            players: HashMap::new(),
        }
    }

    pub fn create_player(&mut self, name: &str) -> u64 {
        self.id += 1;
        let player = Player::new(name.to_string(), self.id);
        self.players.insert(self.id, player);
        self.id
    }

    pub fn create_room(&mut self, name: &str) {
        let room = Room::new(name.to_string());
        self.rooms.insert(name.to_string(), room);
    }
}
