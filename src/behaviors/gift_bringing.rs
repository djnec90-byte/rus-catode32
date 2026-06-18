use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, GiftKind},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Approaching,
    Presenting,
    Satisfied,
}

pub struct GiftBringingBehavior {
    gift: GiftKind,
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
}

impl GiftBringingBehavior {
    pub fn new(gift: GiftKind) -> Self {
        Self {
            gift,
            phase: Phase::Approaching,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 6.0,
            pose_id: PoseId::WalkingSideDetermined,
        }
    }
}

impl Behavior for GiftBringingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::GiftBringing
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Approaching;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 5.0, 8.0);
        self.pose_id = PoseId::WalkingSideDetermined;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Approaching if self.phase_timer >= 1.5 => {
                self.phase = Phase::Presenting;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSideHappy;
            }
            Phase::Presenting if self.phase_timer >= self.total - 1.5 => {
                self.phase = Phase::Satisfied;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSideAloof;
            }
            Phase::Satisfied if self.phase_timer >= 1.5 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus_aff = match self.gift {
            GiftKind::Fish => 7.0,
            GiftKind::Mouse => 5.0,
        };
        let bonus = [
            (StatId::Sociability, 2.0 * progress),
            (StatId::Affection, bonus_aff * progress),
            (StatId::Loyalty, 1.5 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }
}
