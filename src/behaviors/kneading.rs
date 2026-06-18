use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

const KNEAD_POSES: &[PoseId] = &[
    PoseId::KneadingSideNeutral,
    PoseId::KneadingSideHappy,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Kneading,
    Settling,
}

pub struct KneadingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
}

impl KneadingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Kneading,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 8.0,
            pose_id: PoseId::KneadingSideNeutral,
        }
    }
}

impl Behavior for KneadingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Kneading
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Kneading;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 6.0, 12.0);
        self.pose_id = KNEAD_POSES[rand::rand_range_u32(&mut ctx.rng, 0, 1) as usize];
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Kneading if self.phase_timer >= self.total - 2.0 => {
                self.phase = Phase::Settling;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideContent;
            }
            Phase::Settling if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        if rand::rand_f32(&mut rng) < 0.5 {
            Some(NextBehavior::Stretching)
        } else {
            Some(NextBehavior::Lounging)
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 4> = heapless::Vec::new();
        let _ = bonus.push((StatId::Comfort, 5.0));
        let _ = bonus.push((StatId::Serenity, 0.5));
        let _ = bonus.push((StatId::Energy, -1.5));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
