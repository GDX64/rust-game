use serde::{Deserialize, Serialize};

use crate::{
    bullet::Bullet,
    hashgrid::{HashEntity, HashEntityKind},
    utils::vectors::V2D,
};

const CANON_RELOAD_TIME: f64 = 5.0;
pub const SHIP_SIZE: f64 = 10.0;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ShipKey {
    pub id: u64,
    pub player_id: u64,
}

impl ShipKey {
    pub fn new(id: u64, player_id: u64) -> Self {
        Self { id, player_id }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ShipState {
    pub position: V2D,
    pub speed: V2D,
    pub orientation: V2D,
    pub id: u64,
    pub player_id: u64,
    pub cannon_times: [f64; 3],
    pub last_shoot_time: f64,
    pub hp: f64,
}

impl ShipState {
    pub fn key(&self) -> ShipKey {
        ShipKey::new(self.id, self.player_id)
    }

    pub fn to_hash_entity(&self) -> HashEntity {
        let key = self.key();
        HashEntity {
            entity: HashEntityKind::Boat(key),
            position: self.position.into(),
        }
    }
}

impl Default for ShipState {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0).into(),
            speed: (0.0, 0.0).into(),
            orientation: (1.0, 0.0).into(),
            id: 0,
            player_id: 0,
            cannon_times: [0.0, 0.0, 0.0],
            last_shoot_time: 0.0,
            hp: 100.0,
        }
    }
}

impl ShipState {
    pub fn find_available_cannon(&self, current_time: f64) -> Option<usize> {
        for (i, time) in self.cannon_times.iter().enumerate() {
            if current_time - time > CANON_RELOAD_TIME {
                return Some(i);
            }
        }
        None
    }

    pub fn shoot_at(&mut self, current_time: f64, target: V2D) -> Option<Bullet> {
        let cannon_index = self.find_available_cannon(current_time)?;
        let position: V2D = self.position.into();
        let ship_orientation = V2D::from(self.orientation);
        let cannon_multiplier = (cannon_index as i32 - 1) as f64 * SHIP_SIZE / 2.0;
        let cannon_pos = position + ship_orientation * cannon_multiplier;
        self.mark_shoot_time(cannon_index, current_time);

        let bullet = Bullet {
            bullet_id: 0,
            player_id: self.player_id,
            ..Bullet::maybe_from_target(cannon_pos.into(), target.into())?
        };
        self.last_shoot_time = current_time;
        return Some(bullet);
    }

    pub fn mark_shoot_time(&mut self, cannon: usize, current_time: f64) {
        self.cannon_times[cannon] = current_time;
    }
}
