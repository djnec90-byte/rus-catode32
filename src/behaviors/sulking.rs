use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    behaviors::common,
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

const SULK_POSES: &[PoseId] = &[
    PoseId::LayingSideSulking,
    PoseId::LayingSideSulking2,
    PoseId::LayingSideAnnoyed,
    PoseId::LayingSideAngry,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Settling,
    Sulking,
    Emerging,
}

pub struct SulkingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    sulk_pose: PoseId,
}

impl SulkingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Settling,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 14.0,
            pose_id: PoseId::SittingSideAnnoyed,
            sulk_pose: PoseId::LayingSideSulking,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        let stats = [ctx.fullness, ctx.affection, ctx.fulfillment, ctx.comfort];
        let low = stats.iter().filter(|v| **v < 50.0).count();
        ctx.fulfillment < 50.0
            || ctx.affection < 50.0
            || low >= 2
            || stats.iter().any(|v| *v < 25.0)
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let combined = ctx.fulfillment + ctx.affection + ctx.fullness + ctx.comfort;
        let low = [ctx.fullness, ctx.affection, ctx.fulfillment, ctx.comfort]
            .iter()
            .filter(|v| **v < 50.0)
            .count() as f32;
        let ceiling = (combined * 0.225 - low * 5.0).max(10.0);
        let mut base = rand::rand_range_f32(rng, 10.0, ceiling.max(10.0));
        if !ctx.in_familiar_location {
            base *= 0.85;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for SulkingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Sulking
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Settling;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 14.0, 22.0);
        self.sulk_pose = common::pick_pose(&mut ctx.rng, SULK_POSES);
        self.pose_id = PoseId::SittingSideAnnoyed;
        ctx.pending_popup_icon = if ctx.fullness < 30.0 {
            Some("hunger")
        } else if ctx.affection < 30.0 {
            Some("lonely")
        } else {
            Some("sulk")
        };
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Settling if self.phase_timer >= 2.0 => {
                self.phase = Phase::Sulking;
                self.phase_timer = 0.0;
                self.pose_id = self.sulk_pose;
            }
            Phase::Sulking if self.phase_timer >= self.total - 3.0 => {
                self.phase = Phase::Emerging;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSideAnnoyed;
            }
            Phase::Emerging if self.phase_timer >= 2.5 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, _ctx: &GameContext) -> Option<NextBehavior> {
        Some(NextBehavior::Pacing)
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 4> = heapless::Vec::new();
        let _ = bonus.push((StatId::Comfort, 1.5));
        let _ = bonus.push((StatId::Fulfillment, -1.0));
        let _ = bonus.push((StatId::Serenity, -0.5));
        let _ = bonus.push((StatId::Energy, -0.5));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
