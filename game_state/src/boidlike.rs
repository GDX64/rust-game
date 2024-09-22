use cgmath::{InnerSpace, MetricSpace};

use crate::utils::vectors::V2D;

const NEAR: f64 = 40.0;
const TOO_CLOSE: f64 = 20.0;

pub trait BoidLike: PartialEq + Clone {
    fn position(&self) -> V2D;
    fn velocity(&self) -> V2D;
    fn update(&self, speed: V2D) -> Self;

    fn is_near(&self, other: &Self) -> bool {
        let distance = self.position().distance(other.position());
        distance < NEAR && self != other
    }
    fn is_too_close(&self, other: &Self) -> bool {
        let distance = self.position().distance(other.position());
        distance < TOO_CLOSE && self != other
    }
}

pub struct BoidsTeam<B: BoidLike> {
    pub boids: Vec<B>,
}

impl<B: BoidLike> BoidsTeam<B> {
    pub fn update_boids_like(boids: Vec<B>) -> Vec<B> {
        let mut team = BoidsTeam::new(boids);
        team.update();
        team.boids
    }

    fn new(boids: Vec<B>) -> Self {
        Self { boids }
    }

    pub fn update(&mut self) {
        let mut rand_gen = fastrand::Rng::with_seed(0);
        self.boids = self
            .boids
            .iter()
            .map(|boid| {
                let mut repulsion = V2D::new(0.0, 0.0);
                let mut sum_positions = V2D::new(0.0, 0.0);
                let mut total_positions = 0;
                let mut alignment_speed = V2D::new(0.0, 0.0);
                self.boids.iter().for_each(|other| {
                    if !boid.is_near(other) {
                        return;
                    }
                    if boid.is_too_close(other) {
                        let d = (boid.position() - other.position()).normalize();
                        if d.x.is_nan() || d.y.is_nan() {
                            repulsion += V2D::new(rand_gen.f64(), rand_gen.f64()).normalize();
                        } else {
                            repulsion += d;
                        }
                    } else {
                        alignment_speed += other.velocity().normalize();
                    }
                    sum_positions += other.position();
                    total_positions += 1;
                });
                let average_position = if total_positions > 0 {
                    sum_positions / total_positions as f64
                } else {
                    V2D::new(0.0, 0.0)
                };
                let cohesion_speed = average_position - boid.position();

                let result = safe_normalize(repulsion) * 10.0
                    + safe_normalize(alignment_speed) * 1.0
                    + safe_normalize(cohesion_speed) * 1.0;

                if repulsion.magnitude() > 0.0 {
                    boid.update(safe_normalize(result))
                } else {
                    boid.clone()
                }
            })
            .collect();
    }
}

fn safe_normalize(v: V2D) -> V2D {
    if v.x.is_nan() || v.y.is_nan() {
        V2D::new(0.0, 0.0)
    } else {
        v.normalize()
    }
}
