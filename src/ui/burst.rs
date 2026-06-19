//! Spawns the small staggered burst of HEAL sparkles around a screen anchor
//! (used by the medicine action). One group at a time is sufficient for the
//! interactions we support today.

use embedded_graphics::prelude::Point;
use heapless::Vec;

use crate::{
    assets::icons,
    rand,
    render::{Renderer, SpriteOpts},
};

const MAX_PARTICLES: usize = 12;
const BURST_FRAME_DUR: f32 = 0.06;
const BURST_TOTAL: f32 = BURST_FRAME_DUR * 2.0;

#[derive(Clone, Copy)]
struct Particle {
    dx: i16,
    dy: i16,
    delay: f32,
}

pub struct BurstEffect {
    particles: Vec<Particle, MAX_PARTICLES>,
    timer: f32,
}

impl BurstEffect {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            timer: 0.0,
        }
    }

    fn total_duration(&self) -> f32 {
        let max_delay = self
            .particles
            .iter()
            .map(|p| p.delay)
            .fold(0.0_f32, |a, b| a.max(b));
        max_delay + BURST_TOTAL
    }

    /// Spawn `count` HEAL sparkles around (0,0); the anchor is supplied at
    /// draw time so the burst tracks moving characters.
    pub fn trigger_heal(&mut self, rng: &mut u32, count: usize) {
        self.particles.clear();
        self.timer = 0.0;
        let n = count.min(MAX_PARTICLES);
        for i in 0..n {
            let dx = rand::rand_range_f32(rng, -28.0, 28.0) as i16;
            let dy = rand::rand_range_f32(rng, -40.0, -10.0) as i16;
            let jitter = rand::rand_range_f32(rng, 0.0, 0.25);
            let _ = self.particles.push(Particle {
                dx,
                dy,
                delay: i as f32 * 0.18 + jitter,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.particles.is_empty() {
            return;
        }
        self.timer += dt;
        if self.timer >= self.total_duration() {
            self.particles.clear();
            self.timer = 0.0;
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, anchor: Point) {
        if self.particles.is_empty() {
            return;
        }
        let hw = (icons::HEAL_W / 2) as i32;
        let hh = (icons::HEAL_H / 2) as i32;
        for p in &self.particles {
            let elapsed = self.timer - p.delay;
            if elapsed < 0.0 || elapsed >= BURST_TOTAL {
                continue;
            }
            let frame_idx = (elapsed / BURST_FRAME_DUR) as usize;
            let pos = Point::new(
                anchor.x + p.dx as i32 - hw,
                anchor.y + p.dy as i32 - hh,
            );
            // Draw the filled silhouette below the outline so the sparkle
            // punches through whatever is behind it.
            renderer.draw_sprite_raw(
                icons::HEAL_FILL,
                icons::HEAL_W,
                icons::HEAL_H,
                pos,
                SpriteOpts {
                    transparent: true,
                    transparent_color: true,
                    invert: true,
                    frame: frame_idx,
                    ..Default::default()
                },
            );
            renderer.draw_sprite_raw(
                icons::HEAL_FRAME,
                icons::HEAL_W,
                icons::HEAL_H,
                pos,
                SpriteOpts {
                    transparent: true,
                    frame: frame_idx,
                    ..Default::default()
                },
            );
        }
    }
}
