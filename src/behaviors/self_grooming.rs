use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Preparing,
    Grooming,
    Finishing,
}

pub struct SelfGroomingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
}

impl SelfGroomingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Preparing,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 10.0,
            pose_id: PoseId::SittingSideNeutral,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.cleanliness < 57.0 && ctx.energy > 30.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let bulk = rand::rand_range_f32(rng, ctx.cleanliness * 0.5, ctx.cleanliness * 1.5);
        let extra = rand::rand_range_f32(rng, 0.0, (ctx.energy * 0.25).max(10.0));
        (bulk + extra).max(0.0) as u32
    }
}

impl Behavior for SelfGroomingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::SelfGrooming
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
        self.total = rand::rand_range_f32(&mut ctx.rng, 9.0, 14.0);
        self.pose_id = PoseId::SittingSideAloof;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Preparing if self.phase_timer >= 1.5 => {
                self.phase = Phase::Grooming;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingLickingSideLickingLeg;
            }
            Phase::Grooming if self.phase_timer >= self.total - 3.0 => {
                self.phase = Phase::Finishing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSideAloof;
            }
            Phase::Finishing if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 4> = heapless::Vec::new();
        let _ = bonus.push((StatId::Cleanliness, 12.0));
        let _ = bonus.push((StatId::Comfort, 2.0));
        let _ = bonus.push((StatId::Energy, -2.0));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
