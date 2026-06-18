use embedded_graphics::prelude::{Point, Size};

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
    Noticing,
    Listening,
}

pub struct HearingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    icon: Option<&'static str>,
}

impl HearingBehavior {
    pub fn new(icon: Option<&'static str>) -> Self {
        Self {
            phase: Phase::Noticing,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 4.0,
            pose_id: PoseId::SittingForwardShocked,
            icon,
        }
    }
}

impl Behavior for HearingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Hearing
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Noticing;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 3.5, 5.5);
        self.pose_id = PoseId::SittingForwardShocked;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Noticing if self.phase_timer >= 1.0 => {
                self.phase = Phase::Listening;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardNeutral;
            }
            Phase::Listening if self.phase_timer >= self.total - 1.0 => {
                return BehaviorState::Completed
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        let p = ctx.sociability / 200.0;
        if rand::rand_f32(&mut rng) < p {
            Some(NextBehavior::Vocalizing)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        ctx.apply_stat_changes(&[
            (StatId::Sociability, 0.15 * progress),
        ]);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, _: bool) {
        if self.phase != Phase::Noticing {
            return;
        }
        // Question bubble.
        let bx = char_screen.x + 4;
        let by = char_screen.y - 18;
        renderer.draw_rect(Point::new(bx, by), Size::new(12, 10), false);
        renderer.draw_text("?", Point::new(bx + 3, by + 1));
        let _ = self.icon;
    }
}
