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
    listen_duration: f32,
    pose_id: PoseId,
}

impl HearingBehavior {
    pub fn new(_icon: Option<&'static str>) -> Self {
        Self {
            phase: Phase::Noticing,
            phase_timer: 0.0,
            listen_duration: 3.0,
            pose_id: PoseId::SittingForwardShocked,
        }
    }
}

impl Behavior for HearingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Hearing
    }
    fn progress(&self) -> f32 {
        match self.phase {
            Phase::Noticing => 0.0,
            Phase::Listening => (self.phase_timer / self.listen_duration).clamp(0.0, 1.0),
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Noticing;
        self.phase_timer = 0.0;
        self.listen_duration = rand::rand_range_f32(&mut ctx.rng, 2.0, 4.0);
        self.pose_id = PoseId::SittingForwardShocked;
    }

    fn update(&mut self, ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::Noticing => {
                let threshold = rand::rand_range_f32(&mut ctx.rng, 0.5, 1.0);
                if self.phase_timer >= threshold {
                    self.phase = Phase::Listening;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSillySideAloof;
                }
            }
            Phase::Listening if self.phase_timer >= self.listen_duration => {
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
        bubble::draw_above_char(
            renderer,
            BubbleIcon::Question,
            char_screen.x,
            char_screen.y,
            0.0,
            mirror_h,
        );
    }
}
