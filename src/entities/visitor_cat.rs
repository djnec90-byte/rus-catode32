//! Lightweight cat entity rendered as the **other** pet during a
//! playdate. Driven entirely by ESP-NOW `vst ` packets — no behavior
//! manager, no AI ticks, no stat coupling. The visitor's pose and
//! position are network-state; the animation timers and y-coordinate
//! are local.
//!
//! Position extrapolation: every `vst ` frame includes a velocity
//! (`vx`, pixels/second). Between packets [`Self::update`] advances
//! `pos.x` by `vx * dt` so the visitor's walk doesn't pop every 3 s
//! when the heartbeat fires. A fresh `vst ` snaps the position
//! back to authoritative.

use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    character::{draw_pose, PoseAnim},
    render::Renderer,
};

pub struct VisitorCat {
    pub pos: Point,
    pub mirror_h: bool,
    pub pose_id: PoseId,
    /// Most recent velocity from a `vst ` packet, in pixels/second.
    /// Reset to zero when a packet declares the visitor stopped.
    pub vx: f32,
    pub anim: PoseAnim,
}

impl VisitorCat {
    /// Spawn the visitor at the deterministic anchor position. The
    /// caller (`LocationScene::enter` on visit-scene entry) sets
    /// `pos` and `mirror_h` to the side dictated by the visit role,
    /// matching the peer's own setup so the first `vst ` packet is a
    /// no-op rather than a position jump.
    pub fn new(initial_pos: Point) -> Self {
        let pose_id = PoseId::SittingSideNeutral;
        let mut anim = PoseAnim::new(1);
        anim.reseed_for(pose_id.data());
        Self {
            pos: initial_pos,
            mirror_h: false,
            pose_id,
            vx: 0.0,
            anim,
        }
    }

    /// Apply a freshly-received `vst ` packet. Snaps position,
    /// velocity, pose, and facing.
    pub fn apply_state(&mut self, x: i32, pose: PoseId, mirror: bool, vx: f32) {
        self.pos.x = x;
        self.mirror_h = mirror;
        self.vx = vx;
        if self.pose_id != pose {
            self.pose_id = pose;
            self.anim.reseed_for(pose.data());
        }
    }

    /// Extrapolate position between packets and tick the local
    /// animation. `dt` is real seconds.
    pub fn update(&mut self, dt: f32) {
        self.pos.x = (self.pos.x as f32 + self.vx * dt) as i32;
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
            None,
        );
    }
}
