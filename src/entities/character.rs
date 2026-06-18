use embedded_graphics::prelude::Point;
use esp_hal::time::Instant;

use crate::{
    assets::character::PoseId,
    behavior::BehaviorManager,
    character::{draw_pose, PoseAnim},
    context::GameContext,
    render::Renderer,
};

pub struct Character {
    pub pos: Point,
    pub mirror_h: bool,
    pub pose_id: PoseId,
    pub anim: PoseAnim,
    behaviors: BehaviorManager,
}

impl Character {
    pub fn new(pos: Point) -> Self {
        Self {
            pos,
            // TODO: Python randomizes facing direction with random.choice([True, False]) at init.
            mirror_h: false,
            pose_id: PoseId::SittingSideNeutral,
            anim: PoseAnim::new(1),
            behaviors: BehaviorManager::new(),
        }
    }

    pub fn enter(&mut self, ctx: &mut GameContext) {
        let seed = (Instant::now().duration_since_epoch().as_micros() as u32).max(1);
        self.anim = PoseAnim::new(seed);
        self.behaviors.start(ctx);
        self.sync_pose();
        self.anim.reseed_for(self.pose_id.data());
    }

    pub fn update(&mut self, ctx: &mut GameContext, dt: f32) {
        self.behaviors.update(ctx, dt);
        self.sync_pose();
        self.anim.update(self.pose_id.data(), dt);
    }

    pub fn skip_behavior(&mut self, ctx: &mut GameContext) {
        self.behaviors.skip(ctx);
        self.sync_pose();
    }

    pub fn current_behavior_name(&self) -> &'static str {
        self.behaviors.current_name()
    }

    fn sync_pose(&mut self) {
        let next = self.behaviors.current_pose();
        if self.pose_id != next {
            self.pose_id = next;
            self.anim.reseed_for(next.data());
        }
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
