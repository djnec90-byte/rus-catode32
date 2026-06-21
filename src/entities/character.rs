use embedded_graphics::prelude::Point;
use esp_hal::time::Instant;

use crate::{
    assets::{character::PoseId, effects::SWEAT},
    character::{draw_pose, pose_layout, PoseAnim},
    context::GameContext,
    render::{Renderer, SpriteOpts},
};

pub struct Character {
    pub pos: Point,
    pub mirror_h: bool,
    pub pose_id: PoseId,
    pub anim: PoseAnim,
    /// Override for the per-frame eye sprite index; behaviors (currently only
    /// Playing) use this to lock the cat's gaze onto a moving toy.
    pub eye_override: Option<usize>,
    /// Vertical render offset applied at draw time. Negative values lift the
    /// sprite up; used to sink the cat into the cat bed during sleep/nap.
    pub draw_y_offset: i32,
    /// Free-running counter feeding the sick sweat-line frame index.
    sweat_timer: f32,
}

impl Character {
    pub fn new(pos: Point) -> Self {
        Self {
            pos,
            mirror_h: false,
            pose_id: PoseId::SittingSideNeutral,
            anim: PoseAnim::new(1),
            eye_override: None,
            draw_y_offset: 0,
            sweat_timer: 0.0,
        }
    }

    pub fn reseed_anim(&mut self) {
        let seed = (Instant::now().duration_since_epoch().as_micros() as u32).max(1);
        self.anim = PoseAnim::new(seed);
        self.anim.reseed_for(self.pose_id.data());
    }

    pub fn randomize_facing(&mut self, rng: &mut u32) {
        self.mirror_h = crate::rand::rand_bool(rng, 0.5);
    }

    /// Switch to a new pose if it differs from the current one, reseeding the
    /// animation so the new pose's frames start at a randomized offset.
    pub fn set_pose(&mut self, pose: PoseId) {
        if self.pose_id != pose {
            self.pose_id = pose;
            self.anim.reseed_for(pose.data());
        }
    }

    pub fn animate(&mut self, dt: f32) {
        self.anim.update(self.pose_id.data(), dt);
        self.sweat_timer += dt;
    }

    pub fn draw(&self, renderer: &mut Renderer, camera_x: i32) {
        let screen_pos = Point::new(self.pos.x - camera_x, self.pos.y + self.draw_y_offset);
        draw_pose(
            renderer,
            self.pose_id.data(),
            &self.anim,
            screen_pos,
            self.mirror_h,
            self.eye_override,
        );
    }

    /// Draw the sick-sweat wavy lines above the cat's head when sickness has
    /// crossed the visibility threshold (lower while resting, so the player can
    /// still spot it). Anchored to the eye attach point when the pose has eyes,
    /// otherwise to the head attach point — mirrors `CharacterEntity.draw` in
    /// `micropython/src/entities/character.py`.
    pub fn draw_sick_overlay(&self, renderer: &mut Renderer, camera_x: i32, ctx: &GameContext) {
        let threshold = match ctx.current_behavior_name {
            Some("sleeping") | Some("napping") => 2.0,
            _ => 6.0,
        };
        if ctx.sickness < threshold {
            return;
        }
        let pose = self.pose_id.data();
        let screen_pos = Point::new(self.pos.x - camera_x, self.pos.y + self.draw_y_offset);
        let layout = pose_layout(pose, screen_pos, self.mirror_h);
        let (anchor_x, anchor_y) = layout.eye_attach.unwrap_or(layout.head_attach);
        let frame = (self.sweat_timer * 2.0) as usize % SWEAT.frames.len();
        let sx = anchor_x as i32 - SWEAT.width as i32 / 2;
        let sy = anchor_y as i32 - 24;
        renderer.draw_sprite(
            &SWEAT,
            Point::new(sx, sy),
            SpriteOpts {
                frame,
                ..SpriteOpts::default()
            },
        );
    }
}

// Keep `GameContext` accessible from the trait without a circular import.
#[allow(dead_code)]
fn _ctx_marker(_: &GameContext) {}
