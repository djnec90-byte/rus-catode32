use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Approaching,
    Sniffing,
    Reacting,
}

pub struct InvestigatingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
}

impl InvestigatingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Approaching,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 7.0,
            pose_id: PoseId::StandingSideSniffing,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.curiosity >= 40.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let hi = (100.0 - ctx.curiosity).max(10.0);
        let mut base = rand::rand_range_f32(rng, 10.0, hi);
        if !ctx.in_familiar_location {
            base *= 0.8;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for InvestigatingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Investigating
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
        self.total = rand::rand_range_f32(&mut ctx.rng, 6.0, 10.0);
        self.pose_id = PoseId::StandingSideNeutral;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Approaching if self.phase_timer >= 2.0 => {
                self.phase = Phase::Sniffing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::StandingSideSniffing;
            }
            Phase::Sniffing if self.phase_timer >= self.total - 4.0 => {
                self.phase = Phase::Reacting;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardShocked;
            }
            Phase::Reacting if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        if rand::rand_f32(&mut rng) < 0.3 {
            Some(NextBehavior::Observing)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus = [
            (StatId::Focus, -0.25 * progress),
            (StatId::Comfort, -0.2 * progress),
            (StatId::Playfulness, -0.25 * progress),
            (StatId::Serenity, -0.02 * progress),
            (StatId::Curiosity, -0.05 * progress),
            (StatId::Maturity, 0.025 * progress),
            (StatId::Fulfillment, 0.05 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }
}
