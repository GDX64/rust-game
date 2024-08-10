use cgmath::{InnerSpace, MetricSpace};

use crate::game_map::V2D;

const NEAR: f64 = 40.0;

pub trait BoidLike: PartialEq + Clone {
    fn position(&self) -> V2D;
    fn velocity(&self) -> V2D;
    fn update(&self, speed: V2D) -> Self;

    fn is_near(&self, other: &Self) -> bool {
        let distance = self.position().distance(other.position());
        distance < NEAR && self != other
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
                let mut repulsion_speed = V2D::new(0.0, 0.0);
                self.boids
                    .iter()
                    .filter(|other| other.is_near(boid))
                    .for_each(|other| {
                        let d = (boid.position() - other.position()).normalize();
                        if d.x.is_nan() || d.y.is_nan() {
                            repulsion_speed += V2D::new(rand_gen.f64(), rand_gen.f64()).normalize();
                        } else {
                            repulsion_speed += d;
                        }
                    });
                if repulsion_speed.magnitude() > 0.0 {
                    boid.update(repulsion_speed.normalize())
                } else {
                    boid.clone()
                }
            })
            .collect();
    }
}
