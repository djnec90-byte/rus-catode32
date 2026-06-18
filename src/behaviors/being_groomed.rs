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
    Accepting,
    Enjoying,
    Satisfied,
}

pub struct BeingGroomedBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    rejected: bool,
    brush_t: f32,
}

impl BeingGroomedBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Accepting,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 8.0,
            pose_id: PoseId::SittingSideAloof,
            rejected: false,
            brush_t: 0.0,
        }
    }
}

impl Behavior for BeingGroomedBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::BeingGroomed
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.rejected = ctx.affection < 30.0 || ctx.cleanliness > 90.0;
        self.phase = Phase::Accepting;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 7.0, 10.0);
        self.pose_id = if self.rejected {
            PoseId::SittingSideAnnoyed
        } else {
            PoseId::SittingSideAloof
        };
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        self.brush_t += dt;
        match self.phase {
            Phase::Accepting if self.phase_timer >= 1.5 => {
                self.phase = Phase::Enjoying;
                self.phase_timer = 0.0;
                self.pose_id = if self.rejected {
                    PoseId::SittingSideAnnoyed
                } else {
                    PoseId::LayingSideBliss
                };
            }
            Phase::Enjoying if self.phase_timer >= self.total - 2.0 => {
                self.phase = Phase::Satisfied;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideContent;
            }
            Phase::Satisfied if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        if !self.rejected
            && ctx.cleanliness < 70.0
            && ctx.energy > 40.0
            && rand::rand_f32(&mut rng) < 0.4
        {
            Some(NextBehavior::SelfGrooming)
        } else {
            None
        }
    }

    fn exit(&mut self, ctx: &mut GameContext, completed: bool) {
        if completed {
            ctx.milestone_groomed = true;
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mult = if self.rejected { 0.5 } else { 1.0 };
        let bonus = [
            (StatId::Cleanliness, 18.0 * mult * progress),
            (StatId::Affection, 4.0 * mult * progress),
            (StatId::Sociability, 1.5 * mult * progress),
            (StatId::Comfort, 3.0 * mult * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        if !matches!(self.phase, Phase::Enjoying) || self.rejected {
            return;
        }
        bubble::draw_above_char(
            renderer,
            BubbleIcon::Heart,
            char_screen.x,
            char_screen.y,
            self.progress(),
            mirror_h,
        );
    }
}

