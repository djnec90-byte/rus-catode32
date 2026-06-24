use embedded_graphics::prelude::Point;

use crate::{
    assets::{
        character::PoseId,
        icons::{EXCLAIM, EXCLAIM_H, EXCLAIM_W},
    },
    behavior::{AttentionVariant, Behavior, BehaviorId, BehaviorState, NextBehavior},
    behaviors::common,
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::{Renderer, SpriteOpts},
    ui::bubble::{self, BubbleIcon},
};

const PHASE1_DURATION: f32 = 1.5;
const PHASE2_DURATION: f32 = 1.5;
const PHASE3_DURATION: f32 = 2.0;
const REJECTION_DURATION: f32 = 5.0;
const REJECTION_STAT_MULTIPLIER: f32 = 0.5;

const EXCLAIM_RISE_DURATION: f32 = 1.0;
const EXCLAIM_RISE_AMOUNT: i32 = 15;

const REJECTION_POSES: &[PoseId] = &[
    PoseId::StandingSideNeutralLookingDown,
    PoseId::SittingSideLookingDown,
    PoseId::LayingSideNeutral2,
    PoseId::LayingSideBored,
    PoseId::SittingSillySideNeutral,
    PoseId::StandingSideAnnoyed,
    PoseId::LayingSideAnnoyed,
    PoseId::LayingSideContent,
    PoseId::SittingLickingSideLickingLeg,
];

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
    pose_id: PoseId,
    rejected: bool,
}

impl AttentionBehavior {
    pub fn new(variant: AttentionVariant) -> Self {
        Self {
            variant,
            phase: Phase::Noticing,
            phase_timer: 0.0,
            pose_id: PoseId::SittingSillySideNeutral,
            rejected: false,
        }
    }

    /// Product-of-deficits rejection chance: each stat below its threshold
    /// shaves away `(1 - deficit)` from the complement.
    fn rejection_chance(ctx: &GameContext) -> f32 {
        let mut complement = 1.0_f32;
        for (val, threshold) in [
            (ctx.affection, 25.0_f32),
            (ctx.comfort, 30.0),
            (ctx.sociability, 25.0),
            (ctx.courage, 20.0),
        ] {
            if val < threshold {
                let deficit = (threshold - val) / threshold;
                complement *= 1.0 - deficit;
            }
        }
        1.0 - complement
    }
}

impl Behavior for AttentionBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Attention
    }
    fn progress(&self) -> f32 {
        match self.phase {
            Phase::Noticing => (self.phase_timer / PHASE1_DURATION).clamp(0.0, 1.0),
            Phase::Realizing => (self.phase_timer / PHASE2_DURATION).clamp(0.0, 1.0),
            Phase::Happy => (self.phase_timer / PHASE3_DURATION).clamp(0.0, 1.0),
            Phase::Rejecting => (self.phase_timer / REJECTION_DURATION).clamp(0.0, 1.0),
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        // Rejection only happens on "psst", not point_bird.
        self.rejected = matches!(self.variant, AttentionVariant::Psst)
            && rand::rand_f32(&mut ctx.rng) < Self::rejection_chance(ctx);
        self.phase_timer = 0.0;
        if self.rejected {
            self.phase = Phase::Rejecting;
            let i =
                rand::rand_range_u32(&mut ctx.rng, 0, (REJECTION_POSES.len() as u32) - 1) as usize;
            self.pose_id = REJECTION_POSES[i];
        } else {
            self.phase = Phase::Noticing;
            self.pose_id = PoseId::SittingSillySideNeutral;
        }
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::Rejecting if self.phase_timer >= REJECTION_DURATION => {
                return BehaviorState::Completed;
            }
            Phase::Noticing if self.phase_timer >= PHASE1_DURATION => {
                self.phase = Phase::Realizing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSillySideAloof;
            }
            Phase::Realizing if self.phase_timer >= PHASE2_DURATION => {
                self.phase = Phase::Happy;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSillySideHappy;
            }
            Phase::Happy if self.phase_timer >= PHASE3_DURATION => {
                character.play_bursts(&mut ctx.rng, 5);
                return BehaviorState::Completed;
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
            let chance = 0.25 * ((ctx.playfulness + ctx.curiosity) / 100.0);
            let mut rng = ctx.rng;
            if rand::rand_f32(&mut rng) < chance {
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
        let mult = if self.rejected {
            REJECTION_STAT_MULTIPLIER
        } else {
            1.0
        };
        for e in bonus.iter_mut() {
            e.1 *= mult * progress;
        }
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        match self.phase {
            Phase::Noticing => {
                bubble::draw_above_char(
                    renderer,
                    BubbleIcon::Question,
                    char_screen.x,
                    char_screen.y,
                    self.progress(),
                    mirror_h,
                );
            }
            Phase::Realizing => {
                let rise_t = (self.phase_timer / EXCLAIM_RISE_DURATION).min(1.0);
                let rise_offset = (rise_t * EXCLAIM_RISE_AMOUNT as f32) as i32;
                let exclaim_y = char_screen.y - 40 - rise_offset;
                let exclaim_x = if mirror_h {
                    char_screen.x + 16
                } else {
                    char_screen.x - EXCLAIM_W as i32 - 16
                };
                renderer.draw_sprite_raw(
                    EXCLAIM,
                    EXCLAIM_W,
                    EXCLAIM_H,
                    Point::new(exclaim_x, exclaim_y),
                    SpriteOpts::default(),
                );
            }
            _ => {}
        }
    }
}
