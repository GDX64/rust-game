use cgmath::MetricSpace;

use crate::{game_map::V2D, island::Island, player::Player, wasm_game::ServerState};

enum BotState {
    WaitingShips,
    Conquering(Island),
    Reiforcing(Island),
    Dead,
}

const UNITS_PER_ISLAND_TO_ATTACK_AGAIN: usize = 20;
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
            bot_state: BotState::WaitingShips,
            time_to_next_action: 0.0,
        }
    }

    pub fn is_dead(&self) -> bool {
        matches!(self.bot_state, BotState::Dead)
    }

    pub fn tick(&mut self, _dt: f64, game_state: &ServerState) -> Option<()> {
        self.player.tick(&game_state);
        self.player.select_all(&game_state);
        self.player.auto_shoot(&game_state);
        let ships_number = self.player.number_of_ships(&game_state);
        let idle_ships = self.player.number_of_idle_ships(game_state);

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
            BotState::WaitingShips => {
                if ships_number > 0 {
                    let closest_island = self.closes_island_not_mine(game_state)?;
                    self.attack_island(game_state, &closest_island);
                    self.bot_state = BotState::Conquering(closest_island);
                }
            }
            BotState::Conquering(island) => {
                let island = island.clone();
                let units_to_attack =
                    self.my_islands(game_state) * UNITS_PER_ISLAND_TO_ATTACK_AGAIN / 2;
                if let Some(island_dyn) = game_state.island_dynamic.get(&island.id) {
                    let is_mine = island_dyn.owner == Some(self.player.id);
                    if is_mine {
                        self.bot_state = BotState::Reiforcing(island.clone());
                    } else if idle_ships >= units_to_attack {
                        self.attack_island(game_state, &island);
                    }
                }
                if ships_number == 0 {
                    self.bot_state = BotState::Dead;
                }
            }
            BotState::Reiforcing(_island) => {
                let units_to_attack =
                    self.my_islands(game_state) * UNITS_PER_ISLAND_TO_ATTACK_AGAIN;

                if idle_ships >= units_to_attack {
                    let closest_island = self.closes_island_not_mine(game_state)?;
                    self.attack_island(game_state, &closest_island);
                    self.bot_state = BotState::Conquering(closest_island);
                }
                if ships_number == 0 {
                    self.bot_state = BotState::Dead;
                }
            }
            BotState::Dead => {
                return None;
            }
        }
        return None;
    }

    fn attack_island(&mut self, game_state: &ServerState, island: &Island) {
        self.player.select_all_idle(game_state);
        self.player
            .move_selected_ships(game_state, island.light_house.x, island.light_house.y);
    }

    fn my_islands(&self, game_state: &ServerState) -> usize {
        game_state
            .island_dynamic
            .values()
            .filter(|island| return island.owner == Some(self.player.id))
            .count()
    }

    fn closes_island_not_mine(&self, game_state: &ServerState) -> Option<Island> {
        let mut center_of_ships = V2D::new(0.0, 0.0);
        let ships = self.player.my_ships(game_state);
        if ships.len() == 0 {
            return None;
        }
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
                let dist = center_of_ships.distance(pos);
                dist as u64
            });
        return island.cloned();
    }
}
