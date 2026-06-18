use embedded_graphics::prelude::Point;
use esp_hal::time::Instant;

use crate::{
    assets::character::PoseId,
    character::{draw_pose, PoseAnim},
    context::GameContext,
    render::Renderer,
};

pub struct Character {
    pub pos: Point,
    pub mirror_h: bool,
    pub pose_id: PoseId,
    pub anim: PoseAnim,
}

impl Character {
    pub fn new(pos: Point) -> Self {
        Self {
            pos,
            // TODO: Python randomizes facing direction with random.choice([True, False]) at init.
            mirror_h: false,
            pose_id: PoseId::SittingSideNeutral,
            anim: PoseAnim::new(1),
        }
    }

    pub fn reseed_anim(&mut self) {
        let seed = (Instant::now().duration_since_epoch().as_micros() as u32).max(1);
        self.anim = PoseAnim::new(seed);
        self.anim.reseed_for(self.pose_id.data());
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
    }

    pub fn draw(&self, renderer: &mut Renderer, camera_x: i32) {
        let screen_pos = Point::new(self.pos.x - camera_x, self.pos.y);
        draw_pose(
            renderer,
            self.pose_id.data(),
            &self.anim,
            screen_pos,
            self.mirror_h,
        );
    }
}

// Keep `GameContext` accessible from the trait without a circular import.
#[allow(dead_code)]
fn _ctx_marker(_: &GameContext) {}
