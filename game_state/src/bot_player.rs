use cgmath::InnerSpace;

use crate::{
    game_map::{Island, V2D},
    player::Player,
    wasm_game::ServerState,
};

enum BotState {
    WaitingShips,
    Idle,
    Conquering(Island),
    Reiforcing(Island),
}

const UNITS_TO_ATTACK_AGAIN: usize = 20;

pub struct BotPlayer {
    pub player: Player,
    bot_state: BotState,
}

impl BotPlayer {
    pub fn new(id: u64) -> Self {
        Self {
            player: Player::new(id),
            bot_state: BotState::Idle,
        }
    }

    pub fn tick(&mut self, _dt: f64, game_state: &ServerState) -> Option<()> {
        self.player.tick(&game_state);
        self.player.select_all(&game_state);
        self.player.auto_shoot(&game_state);
        let ships_number = self.player.number_of_ships(&game_state);
        if ships_number <= 0 {
            self.bot_state = BotState::WaitingShips;
            let max_size = game_state.game_map.dim;
            let x = (self.player.rng.f64() - 0.5) * max_size;
            let y = (self.player.rng.f64() - 0.5) * max_size;
            for _ in 0..20 {
                self.player.create_ship(x, y)
            }
        }

        match &self.bot_state {
            BotState::WaitingShips => {
                if self.player.number_of_ships(&game_state) > 0 {
                    self.bot_state = BotState::Idle;
                }
            }
            BotState::Idle => {
                let closest_island = self.attack_closest_island(game_state)?;
                self.bot_state = BotState::Conquering(closest_island);
            }
            BotState::Conquering(island) => {
                if let Some(island_dyn) = game_state.island_dynamic.get(&island.id) {
                    let is_mine = island_dyn.owner == Some(self.player.id);
                    if is_mine {
                        self.bot_state = BotState::Reiforcing(island.clone());
                    }
                }
            }
            BotState::Reiforcing(_island) => {
                if ships_number < UNITS_TO_ATTACK_AGAIN {
                    return None;
                }
                let closest_island = self.attack_closest_island(game_state)?;
                self.bot_state = BotState::Conquering(closest_island);
            }
        }
        return None;
    }

    fn attack_closest_island(&mut self, game_state: &ServerState) -> Option<Island> {
        let closest_island = self.closes_island_not_mine(game_state);
        if let Some(island) = closest_island {
            self.player.select_all(game_state);
            self.player
                .move_selected_ships(game_state, island.light_house.x, island.light_house.y);
            return Some(island);
        }
        return None;
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
