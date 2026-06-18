use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Chattering,
    Settling,
}

pub struct ChatteringBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    chatter_t: f32,
}

impl ChatteringBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Chattering,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 6.0,
            pose_id: PoseId::SittingForwardShocked,
            chatter_t: 0.0,
        }
    }
}

impl Behavior for ChatteringBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Chattering
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Chattering;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 5.0, 9.0);
        self.pose_id = PoseId::YellingForwardLiftAndYell;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        self.chatter_t += dt;
        match self.phase {
            Phase::Chattering if self.phase_timer >= self.total - 1.5 => {
                self.phase = Phase::Settling;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardAloof;
            }
            Phase::Settling if self.phase_timer >= 1.5 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, _ctx: &GameContext) -> Option<NextBehavior> {
        Some(NextBehavior::Observing)
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 3> = heapless::Vec::new();
        let _ = bonus.push((StatId::Focus, -2.0));
        let _ = bonus.push((StatId::Curiosity, -1.0));
        let _ = bonus.push((StatId::Sociability, 0.3));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, _: bool) {
        if self.phase != Phase::Chattering {
            return;
        }
        // Three stacked pulsing "ek!" texts above the head.
        let pulse = ((self.chatter_t * 6.0) as i32) % 3;
        for i in 0..(pulse + 1) {
            renderer.draw_text(
                "ek!",
                Point::new(char_screen.x - 10, char_screen.y - 18 - i * 8),
            );
        }
    }
}
