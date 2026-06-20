//! Reusable staggered burst of sparkle particles around a screen anchor.
//!
//! Direct port of `ui.py::BurstEffect`. By default each particle animates
//! through the 5-frame `BURST1` sparkle at 8 fps (0.625s per particle), with a
//! 0.5s stagger between particles plus a small jitter — matching the pacing
//! the Python game has shipped with since launch.

use embedded_graphics::prelude::Point;
use heapless::Vec;

use crate::{
    assets::{effects::{BURST1, BURST1_FRAME_DUR}, icons},
    rand,
    render::{Renderer, Sprite, SpriteOpts},
};

const MAX_PARTICLES: usize = 12;

#[derive(Clone, Copy)]
struct Particle {
    dx: i16,
    dy: i16,
    delay: f32,
}

/// Visual style for a burst. `frames` is a slice of sprite frames played at
/// `frame_dur` seconds each. `fill_frames` is an optional inverted-fill mask
/// (used by the HEAL sparkle to punch through whatever is behind it).
#[derive(Clone, Copy)]
pub struct BurstStyle {
    pub frames: &'static [&'static [u8]],
    pub fill_frames: Option<&'static [&'static [u8]]>,
    pub width: u16,
    pub height: u16,
    pub frame_dur: f32,
}

impl BurstStyle {
    pub const fn from_sprite(sprite: &'static Sprite, frame_dur: f32) -> Self {
        Self {
            frames: sprite.frames,
            fill_frames: sprite.fill_frames,
            width: sprite.width,
            height: sprite.height,
            frame_dur,
        }
    }

    fn total(&self) -> f32 {
        self.frames.len() as f32 * self.frame_dur
    }
}

/// Default sparkle: BURST1 played at 8 fps. Used for watering / fertilizing
/// plant feedback (and anywhere else that just wants the generic sparkle).
pub const DEFAULT_STYLE: BurstStyle = BurstStyle::from_sprite(&BURST1, BURST1_FRAME_DUR);

/// HEAL sparkle (medicine action). 7x7, single outlined frame backed by a
/// solid fill that punches through whatever is behind.
pub const HEAL_STYLE: BurstStyle = BurstStyle {
    frames: HEAL_FRAMES,
    fill_frames: Some(HEAL_FILL_FRAMES),
    width: icons::HEAL_W,
    height: icons::HEAL_H,
    frame_dur: 0.06,
};

const HEAL_FRAMES: &[&[u8]] = &[icons::HEAL_FRAME];
const HEAL_FILL_FRAMES: &[&[u8]] = &[icons::HEAL_FILL];

pub struct BurstEffect {
    particles: Vec<Particle, MAX_PARTICLES>,
    timer: f32,
    style: BurstStyle,
}

impl BurstEffect {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            timer: 0.0,
            style: DEFAULT_STYLE,
        }
    }

    pub fn active(&self) -> bool {
        !self.particles.is_empty()
    }

    fn total_duration(&self) -> f32 {
        let max_delay = self
            .particles
            .iter()
            .map(|p| p.delay)
            .fold(0.0_f32, |a, b| a.max(b));
        max_delay + self.style.total()
    }

    /// HEAL sparkles around the character — wide spread, drifting upward.
    /// Mirrors Python `character.play_bursts(count=10, icon=HEAL,
    /// spread_x=28, spread_y_min=-40, spread_y_max=-10)`.
    pub fn trigger_heal(&mut self, rng: &mut u32, count: usize) {
        self.trigger_with(rng, count, HEAL_STYLE, -28.0, 28.0, -40.0, -10.0, 0.5);
    }

    /// Plant watering / fertilizer feedback — narrow spread centered just
    /// above the plant. Mirrors the Python `_e.trigger(count=4, spread_x=12,
    /// spread_y_min=-25, spread_y_max=5)` call from `tend_water` /
    /// `tend_fertilize` (default BURST1 sparkle, no icon override).
    pub fn trigger_plant(&mut self, rng: &mut u32, count: usize) {
        self.trigger_with(rng, count, DEFAULT_STYLE, -12.0, 12.0, -25.0, 5.0, 0.5);
    }

    pub fn trigger_with(
        &mut self,
        rng: &mut u32,
        count: usize,
        style: BurstStyle,
        spread_x_min: f32,
        spread_x_max: f32,
        spread_y_min: f32,
        spread_y_max: f32,
        stagger: f32,
    ) {
        self.particles.clear();
        self.timer = 0.0;
        self.style = style;
        let n = count.min(MAX_PARTICLES);
        for i in 0..n {
            let dx = rand::rand_range_f32(rng, spread_x_min, spread_x_max) as i16;
            let dy = rand::rand_range_f32(rng, spread_y_min, spread_y_max) as i16;
            // Python uses `i * 0.5 + uniform(0.0, 0.25)`; keep the jitter
            // proportional so non-default stagger values still scale.
            let jitter = rand::rand_range_f32(rng, 0.0, stagger * 0.5);
            let _ = self.particles.push(Particle {
                dx,
                dy,
                delay: i as f32 * stagger + jitter,
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
        let hw = (self.style.width / 2) as i32;
        let hh = (self.style.height / 2) as i32;
        let total = self.style.total();
        let n_frames = self.style.frames.len();
        for p in &self.particles {
            let elapsed = self.timer - p.delay;
            if elapsed < 0.0 || elapsed >= total {
                continue;
            }
            let frame_idx = ((elapsed / self.style.frame_dur) as usize).min(n_frames - 1);
            let pos = Point::new(
                anchor.x + p.dx as i32 - hw,
                anchor.y + p.dy as i32 - hh,
            );
            if let Some(fill_frames) = self.style.fill_frames {
                let fi = frame_idx.min(fill_frames.len() - 1);
                renderer.draw_sprite_raw(
                    fill_frames[fi],
                    self.style.width,
                    self.style.height,
                    pos,
                    SpriteOpts {
                        transparent: true,
                        transparent_color: true,
                        invert: true,
                        ..Default::default()
                    },
                );
            }
            renderer.draw_sprite_raw(
                self.style.frames[frame_idx],
                self.style.width,
                self.style.height,
                pos,
                SpriteOpts {
                    transparent: true,
                    ..Default::default()
                },
            );
        }
    }
}
