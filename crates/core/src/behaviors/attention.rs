use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{AttentionVariant, Behavior, BehaviorId, BehaviorState, NextBehavior},
    behaviors::common,
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
    ui::bubble::{self, BubbleIcon},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Noticing,
    Realizing,
    Happy,
    Rejecting,
}

pub struct AttentionBehavior {
    variant: AttentionVariant,
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    rejected: bool,
    excl_rise: f32,
}

impl AttentionBehavior {
    pub fn new(variant: AttentionVariant) -> Self {
        Self {
            variant,
            phase: Phase::Noticing,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 6.0,
            pose_id: PoseId::SittingSideLookingDown,
            rejected: false,
            excl_rise: 0.0,
        }
    }
}

impl Behavior for AttentionBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Attention
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.rejected = ctx.affection < 25.0 && rand::rand_bool(&mut ctx.rng, 0.5);
        self.phase = if self.rejected {
            Phase::Rejecting
        } else {
            Phase::Noticing
        };
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 5.0, 8.0);
        self.pose_id = if self.rejected {
            PoseId::SittingSideAnnoyed
        } else {
            PoseId::SittingSideLookingDown
        };
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        self.excl_rise += dt;
        match self.phase {
            Phase::Rejecting if self.phase_timer >= 2.5 => return BehaviorState::Completed,
            Phase::Noticing if self.phase_timer >= 1.5 => {
                self.phase = Phase::Realizing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardShocked;
            }
            Phase::Realizing if self.phase_timer >= 1.5 => {
                self.phase = Phase::Happy;
                self.phase_timer = 0.0;
                self.pose_id = match self.variant {
                    AttentionVariant::Psst => PoseId::SittingForwardHappy,
                    AttentionVariant::PointBird => PoseId::SittingSideHappy,
                };
            }
            Phase::Happy if self.phase_timer >= self.total - 3.0 => {
                return BehaviorState::Completed
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        if self.rejected {
            return Some(NextBehavior::Meandering);
        }
        if matches!(self.variant, AttentionVariant::PointBird) {
            let mut rng = ctx.rng;
            if rand::rand_f32(&mut rng) < 0.4 {
                return Some(NextBehavior::Chattering);
            }
        }
        None
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
        match self.variant {
            AttentionVariant::Psst => {
                common::bonus_add(&mut bonus, StatId::Curiosity, 3.0);
                common::bonus_add(&mut bonus, StatId::Playfulness, 1.5);
                common::bonus_add(&mut bonus, StatId::Focus, 3.0);
                common::bonus_add(&mut bonus, StatId::Courage, 0.05);
                common::bonus_add(&mut bonus, StatId::Intelligence, 0.5);
            }
            AttentionVariant::PointBird => {
                common::bonus_add(&mut bonus, StatId::Curiosity, 5.0);
                common::bonus_add(&mut bonus, StatId::Playfulness, 2.5);
                common::bonus_add(&mut bonus, StatId::Focus, 2.0);
                common::bonus_add(&mut bonus, StatId::Courage, 0.05);
                common::bonus_add(&mut bonus, StatId::Intelligence, 0.5);
            }
        }
        let mult = if self.rejected { 0.5 } else { 1.0 };
        for e in bonus.iter_mut() {
            e.1 *= mult * progress;
        }
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        if matches!(self.phase, Phase::Realizing) {
            // Rising exclaim above head.
            let rise = ((self.excl_rise * 14.0) as i32).min(14);
            renderer.draw_text(
                "!",
                Point::new(char_screen.x - 2, char_screen.y - 14 - rise),
            );
        } else if matches!(self.phase, Phase::Noticing) {
            bubble::draw_above_char(
                renderer,
                BubbleIcon::Question,
                char_screen.x,
                char_screen.y,
                self.progress(),
                mirror_h,
            );
        }
    }
}
