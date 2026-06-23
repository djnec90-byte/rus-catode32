use embedded_graphics::prelude::Point;

use crate::{
    assets::nature::{BUTTERFLY1, FIREFLY, MOTH},
    rand::{rand_bool, rand_range_f32, rand_range_u32, xorshift32},
    render::{Renderer, Sprite, SpriteOpts},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FlyerKind {
    Butterfly,
    Moth,
    Firefly,
}

fn sprite_for(kind: FlyerKind) -> &'static Sprite {
    match kind {
        FlyerKind::Butterfly => &BUTTERFLY1,
        FlyerKind::Moth => &MOTH,
        FlyerKind::Firefly => &FIREFLY,
    }
}

pub struct FlyerEntity {
    pub kind: FlyerKind,
    pub x: f32,
    pub y: f32,
    sprite: &'static Sprite,
    anim_counter: f32,
    anim_speed: f32,
    vx: f32,
    vy: f32,
    bounds_left: f32,
    bounds_right: f32,
    bounds_top: f32,
    bounds_bottom: f32,
    direction_timer: f32,
    direction_interval: f32,
    rng: u32,
}

impl FlyerEntity {
    pub fn new(kind: FlyerKind, x: f32, y: f32, world_width: i32, parent_rng: &mut u32) -> Self {
        let sprite = sprite_for(kind);
        // Firefly has a fixed speed (2 fps); the others get a random 7-12 fps.
        let mut local_rng = xorshift32(parent_rng);
        let anim_speed = if kind == FlyerKind::Firefly {
            2.0
        } else {
            rand_range_u32(&mut local_rng, 7, 12) as f32
        };
        Self {
            kind,
            x,
            y,
            sprite,
            anim_counter: 0.0,
            anim_speed,
            vx: 0.5,
            vy: 0.3,
            bounds_left: 10.0,
            bounds_right: (world_width - 10) as f32,
            bounds_top: 10.0,
            bounds_bottom: 45.0,
            direction_timer: 0.0,
            direction_interval: 2.0,
            rng: local_rng,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let frame_count = self.sprite.frames.len() as f32;
        self.anim_counter = (self.anim_counter + dt * self.anim_speed) % frame_count;

        self.direction_timer += dt;
        if self.direction_timer >= self.direction_interval {
            self.direction_timer = 0.0;
            self.pick_new_direction();
        }

        self.x += self.vx * dt * 20.0;
        self.y += self.vy * dt * 20.0;

        if self.x < self.bounds_left {
            self.x = self.bounds_left;
            self.vx = self.vx.abs();
        } else if self.x > self.bounds_right {
            self.x = self.bounds_right;
            self.vx = -self.vx.abs();
        }
        if self.y < self.bounds_top {
            self.y = self.bounds_top;
            self.vy = self.vy.abs();
        } else if self.y > self.bounds_bottom {
            self.y = self.bounds_bottom;
            self.vy = -self.vy.abs();
        }
    }

    fn pick_new_direction(&mut self) {
        if rand_bool(&mut self.rng, 0.5) {
            self.vy = rand_range_f32(&mut self.rng, -0.5, 0.5);
        }
        if rand_bool(&mut self.rng, 0.3) {
            self.vx = rand_range_f32(&mut self.rng, -0.5, 0.5);
        }
        self.direction_interval = rand_range_f32(&mut self.rng, 1.0, 3.0);
    }

    pub fn draw(&self, renderer: &mut Renderer, camera_offset: i32) {
        let frame = (self.anim_counter as usize) % self.sprite.frames.len();
        renderer.draw_sprite(
            self.sprite,
            Point::new(self.x as i32 - camera_offset, self.y as i32),
            SpriteOpts {
                frame,
                ..Default::default()
            },
        );
    }
}
