use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};

use crate::game_map::{V2D, V3D};

const BULLET_SPEED: f64 = 150.0;
const GRAVITY: f64 = 9.81;
const MAX_SHOOT_ANGLE: f64 = 3.14 / 180.0 * 10.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Bullet {
    pub position: (f64, f64, f64),
    pub speed: (f64, f64, f64),
    pub player_id: u64,
    pub bullet_id: u64,
    pub target: (f64, f64, f64),
    pub time: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BulletSnapShot {
    pub position: (f64, f64, f64),
    pub player_id: u64,
    pub bullet_id: u64,
}

impl Bullet {
    pub fn max_distance() -> f64 {
        let v_horizontal = BULLET_SPEED * f64::cos(MAX_SHOOT_ANGLE);
        let v_vertical = BULLET_SPEED * f64::sin(MAX_SHOOT_ANGLE);
        let time = v_vertical * 2.0 / GRAVITY;
        let distance = v_horizontal * time;
        return distance;
    }

    pub fn maybe_from_target(initial: V2D, target: V2D) -> Option<Bullet> {
        let v0 = BULLET_SPEED;
        let g = GRAVITY;
        let initial: V3D = (initial.x, initial.y, 0.0).into();
        let target: V3D = (target.x, target.y, 0.0).into();
        let d_vector = target - initial;
        let d = d_vector.magnitude();
        let angle = f64::asin(d * g / (2.0 * v0 * v0));
        let angle = if angle.is_nan() || angle > MAX_SHOOT_ANGLE {
            return None;
        } else {
            angle
        };
        let vxy = v0 * f64::cos(angle);
        let vx = d_vector.normalize() * vxy;

        //due to numerical errors on the angle calc,
        //we may not hit the target
        //so we calculate the time and adjust the z speed
        let end_time = d / vxy;
        let vz = GRAVITY * end_time / 2.0;

        let speed = (vx.x, vx.y, vz).into();

        return Some(Bullet {
            position: initial.into(),
            speed,
            player_id: 0,
            bullet_id: 0,
            target: target.into(),
            time: 0.0,
        });
    }

    pub fn snapshot(&self) -> BulletSnapShot {
        BulletSnapShot {
            position: self.current_pos().into(),
            player_id: self.player_id,
            bullet_id: self.bullet_id,
        }
    }

    pub fn final_pos(&self) -> V3D {
        self.eval(self.end_time())
    }

    pub fn current_pos(&self) -> V3D {
        self.eval(self.time)
    }

    fn eval(&self, t: f64) -> V3D {
        let pos: V3D = self.position.into();
        let speed: V3D = self.speed.into();
        pos + speed * t + V3D::new(0.0, 0.0, -GRAVITY * t * t / 2.0)
    }

    fn end_time(&self) -> f64 {
        let z_speed = self.speed.2;
        let time = z_speed / GRAVITY * 2.0;
        return time;
    }

    pub fn evolve(&mut self, dt: f64) {
        self.time += dt;
    }

    pub fn is_finished(&self) -> bool {
        self.time > self.end_time()
    }
}

#[cfg(test)]
mod test {
    use cgmath::MetricSpace;

    use super::Bullet;
    const BLAST_RADIUS: f64 = 1.0;

    fn verify_hits_target(initial: (f64, f64), target: (f64, f64)) -> bool {
        let bullet = Bullet::maybe_from_target(initial.into(), target.into()).unwrap();
        let hit = bullet.final_pos();
        println!("{:?}", hit);
        return hit.distance((target.0, target.1, 0.0).into()) < BLAST_RADIUS;
    }

    #[test]
    fn test_shoot() {
        assert!(verify_hits_target((0.0, 0.0), (5.0, 5.0)));
        assert!(verify_hits_target((0.0, 0.0), (-5.0, 0.0)));
        assert!(verify_hits_target((0.0, 0.0), (0.0, 3.0)));
        assert!(verify_hits_target((0.0, 0.0), (1000.0, 1000.0)));
    }
}
