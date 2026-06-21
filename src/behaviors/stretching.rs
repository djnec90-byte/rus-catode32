use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Preparing,
    Stretching,
    Relaxing,
}

pub struct StretchingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
}

impl StretchingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Preparing,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 6.0,
            pose_id: PoseId::SittingSideNeutral,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.comfort < 55.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let lo = ctx.comfort * 0.4;
        let hi = ctx.comfort.max(10.0);
        rand::rand_range_f32(rng, lo, hi).max(0.0) as u32
    }
}

impl Behavior for StretchingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Stretching
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Preparing;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 5.0, 9.0);
        self.pose_id = PoseId::SittingSideNeutral;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Preparing if self.phase_timer >= 1.5 => {
                self.phase = Phase::Stretching;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LeaningForwardSideStretch;
            }
            Phase::Stretching if self.phase_timer >= self.total - 4.5 => {
                self.phase = Phase::Relaxing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSideAloof;
            }
            Phase::Relaxing if self.phase_timer >= 1.5 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        if rand::rand_f32(&mut rng) < 0.2 {
            Some(NextBehavior::Kneading)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        ctx.apply_stat_changes(&[(StatId::Comfort, 3.0 * progress)]);
    }
}
