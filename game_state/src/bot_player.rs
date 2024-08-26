use cgmath::InnerSpace;
use log::info;

use crate::{
    game_map::{Island, V2D},
    player::Player,
    wasm_game::ServerState,
};

enum BotState {
    Idle,
    Conquering(Island),
    Reiforcing(Island),
}

const UNITS_TO_ATTACK_AGAIN: usize = 20;
const TIME_FOR_ACTION: f64 = 1.0;

pub struct BotPlayer {
    pub player: Player,
    bot_state: BotState,
    time_to_next_action: f64,
}

impl BotPlayer {
    pub fn new(id: u64) -> Self {
        Self {
            player: Player::new(id),
            bot_state: BotState::Idle,
            time_to_next_action: 0.0,
        }
    }

    pub fn tick(&mut self, _dt: f64, game_state: &ServerState) -> Option<()> {
        self.player.tick(&game_state);
        self.player.select_all(&game_state);
        self.player.auto_shoot(&game_state);
        let ships_number = self.player.number_of_ships(&game_state);
        if ships_number <= 0 {
            self.bot_state = BotState::Idle;
            let max_size = game_state.game_map.dim;
            let x = (self.player.rng.f64() - 0.5) * max_size / 2.0;
            let y = (self.player.rng.f64() - 0.5) * max_size / 2.0;
            for _ in 0..20 {
                self.player.create_ship(x, y)
            }
        }

        let current_time = game_state.current_time;
        if current_time < self.time_to_next_action {
            return None;
        }
        self.time_to_next_action = current_time + TIME_FOR_ACTION;
        let should_take_action = self.player.rng.f64() < 0.5;
        if !should_take_action {
            return None;
        }

        match &self.bot_state {
            BotState::Idle => {
                let closest_island = self.closes_island_not_mine(game_state)?;
                self.attack_island(game_state, &closest_island);
                self.bot_state = BotState::Conquering(closest_island);
            }
            BotState::Conquering(island) => {
                let island = island.clone();
                if let Some(island_dyn) = game_state.island_dynamic.get(&island.id) {
                    let is_mine = island_dyn.owner == Some(self.player.id);
                    if is_mine {
                        self.bot_state = BotState::Reiforcing(island.clone());
                    } else {
                        self.attack_island(game_state, &island);
                    }
                }
            }
            BotState::Reiforcing(_island) => {
                if ships_number < UNITS_TO_ATTACK_AGAIN {
                    let closest_island = self.closes_island_not_mine(game_state)?;
                    self.attack_island(game_state, &closest_island);
                    self.bot_state = BotState::Conquering(closest_island);
                }
            }
        }
        info!("Bot state: {:?}", self.bot_state);

        return None;
    }

    fn attack_island(&mut self, game_state: &ServerState, island: &Island) {
        self.player.select_all(game_state);
        self.player
            .move_selected_ships(game_state, island.light_house.x, island.light_house.y);
    }

    fn closes_island_not_mine(&self, game_state: &ServerState) -> Option<Island> {
        let mut center_of_ships = V2D::new(0.0, 0.0);
        let ships = self.player.my_ships(game_state);
        for ship in ships.iter() {
            center_of_ships += ship.position.into();
        }
        center_of_ships /= ships.len() as f64;
        let island = game_state
            .island_dynamic
            .values()
            .filter_map(|island| {
                if island.owner == Some(self.player.id) {
                    return None;
                }
                return game_state.game_map.islands.get(&island.id);
            })
            .min_by_key(|&island| {
                let pos = island.light_house;
                let dist = (center_of_ships - pos).magnitude();
                dist as i32
            });
        return island.cloned();
    }
}
