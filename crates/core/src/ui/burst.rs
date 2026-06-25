//! Reusable staggered burst of sparkle particles around a screen anchor.
//!
//! By default each particle animates through the 5-frame `BURST1` sparkle at
//! 8 fps (0.625s per particle), with a 0.5s stagger between particles plus a
//! small jitter. Multiple groups can be active concurrently — each call to
//! `trigger_*` adds an independent group anchored at the draw-time `base`.

use embedded_graphics::prelude::Point;
use heapless::Vec;

use crate::{
    assets::{effects::{BURST1, BURST1_FRAME_DUR}, icons},
    rand,
    render::{Renderer, Sprite, SpriteOpts},
};

const MAX_PARTICLES_PER_GROUP: usize = 12;
const MAX_GROUPS: usize = 4;

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
    frame_dur: 0.5,
};

const HEAL_FRAMES: &[&[u8]] = &[icons::HEAL_FRAME];
const HEAL_FILL_FRAMES: &[&[u8]] = &[icons::HEAL_FILL];

#[derive(Clone)]
struct Group {
    particles: Vec<Particle, MAX_PARTICLES_PER_GROUP>,
    timer: f32,
    style: BurstStyle,
}

impl Group {
    fn total_duration(&self) -> f32 {
        let max_delay = self
            .particles
            .iter()
            .map(|p| p.delay)
            .fold(0.0_f32, |a, b| a.max(b));
        max_delay + self.style.total()
    }

    fn expired(&self) -> bool {
        self.timer >= self.total_duration()
    }
}

pub struct BurstEffect {
    groups: Vec<Group, MAX_GROUPS>,
}

impl BurstEffect {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
        }
    }

    pub fn active(&self) -> bool {
        !self.groups.is_empty()
    }

    /// Character-style spread (Python default): dx in [-35, 35], dy in [-50,
    /// -20], stagger 0.5s + up to 0.25s jitter.
    pub fn trigger_character(&mut self, rng: &mut u32, count: usize, style: BurstStyle) {
        self.trigger_with(rng, count, style, -35.0, 35.0, -50.0, -20.0, 0.5);
    }

    /// Plant watering / fertilizer feedback: narrow spread centered just
    /// above the plant.
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
        // Drop the oldest still-running group if we're at capacity, so a fresh
        // call always lands.
        if self.groups.len() == MAX_GROUPS {
            self.groups.remove(0);
        }
        let mut group = Group {
            particles: Vec::new(),
            timer: 0.0,
            style,
        };
        let n = count.min(MAX_PARTICLES_PER_GROUP);
        for i in 0..n {
            let dx = rand::rand_range_f32(rng, spread_x_min, spread_x_max) as i16;
            let dy = rand::rand_range_f32(rng, spread_y_min, spread_y_max) as i16;
            // Keep the jitter proportional so non-default stagger values
            // still scale.
            let jitter = rand::rand_range_f32(rng, 0.0, stagger * 0.5);
            let _ = group.particles.push(Particle {
                dx,
                dy,
                delay: i as f32 * stagger + jitter,
            });
        }
        let _ = self.groups.push(group);
    }

    pub fn update(&mut self, dt: f32) {
        if self.groups.is_empty() {
            return;
        }
        for g in self.groups.iter_mut() {
            g.timer += dt;
        }
        // Retain only groups that still have remaining lifetime.
        let mut i = 0;
        while i < self.groups.len() {
            if self.groups[i].expired() {
                self.groups.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn draw(&self, renderer: &mut Renderer, anchor: Point) {
        if self.groups.is_empty() {
            return;
        }
        for g in &self.groups {
            let hw = (g.style.width / 2) as i32;
            let hh = (g.style.height / 2) as i32;
            let total = g.style.total();
            let n_frames = g.style.frames.len();
            for p in &g.particles {
                let elapsed = g.timer - p.delay;
                if elapsed < 0.0 || elapsed >= total {
                    continue;
                }
                let frame_idx = ((elapsed / g.style.frame_dur) as usize).min(n_frames - 1);
                let pos = Point::new(
                    anchor.x + p.dx as i32 - hw,
                    anchor.y + p.dy as i32 - hh,
                );
                if let Some(fill_frames) = g.style.fill_frames {
                    let fi = frame_idx.min(fill_frames.len() - 1);
                    renderer.draw_sprite_raw(
                        fill_frames[fi],
                        g.style.width,
                        g.style.height,
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
                    g.style.frames[frame_idx],
                    g.style.width,
                    g.style.height,
                    pos,
                    SpriteOpts {
                        transparent: true,
                        ..Default::default()
                    },
                );
            }
        }
    }
}
