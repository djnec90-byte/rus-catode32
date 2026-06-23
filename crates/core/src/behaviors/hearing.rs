use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
    ui::bubble::{self, BubbleIcon},
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
        if ctx.sociability > 20.0 {
            let mut p = 0.7 + (ctx.sociability - 20.0) / 267.0;
            if p > 0.95 {
                p = 0.95;
            }
            let mut rng = ctx.rng;
            if rand::rand_f32(&mut rng) < p {
                return Some(NextBehavior::Vocalizing);
            }
        }
        None
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        ctx.apply_stat_changes(&[
            (StatId::Sociability, 0.2 * progress),
        ]);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        if self.phase != Phase::Noticing {
            return;
        }
        let icon = self
            .icon
            .and_then(BubbleIcon::from_name)
            .unwrap_or(BubbleIcon::Question);
        bubble::draw_above_char(
            renderer,
            icon,
            char_screen.x,
            char_screen.y,
            0.0,
            mirror_h,
        );
    }
}
