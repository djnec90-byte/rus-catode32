use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Noticing,
    Watching,
    LosingInterest,
}

pub struct ObservingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
}

impl ObservingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Noticing,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 10.0,
            pose_id: PoseId::SittingForwardNeutral,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.curiosity >= 30.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let hi = (100.0 - ctx.curiosity).max(10.0);
        rand::rand_range_f32(rng, 10.0, hi).max(0.0) as u32
    }
}

impl Behavior for ObservingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Observing
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Noticing;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 8.0, 14.0);
        self.pose_id = PoseId::SittingForwardShocked;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Noticing if self.phase_timer >= 1.5 => {
                self.phase = Phase::Watching;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardNeutral;
            }
            Phase::Watching if self.phase_timer >= self.total - 2.5 => {
                self.phase = Phase::LosingInterest;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardAloof;
            }
            Phase::LosingInterest if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        let r = rand::rand_f32(&mut rng);
        if ctx.playfulness > 50.0 && r < 0.4 {
            Some(NextBehavior::Chattering)
        } else if ctx.focus > 40.0 && r < 0.7 {
            Some(NextBehavior::Investigating)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 4> = heapless::Vec::new();
        let _ = bonus.push((StatId::Focus, -0.5));
        let _ = bonus.push((StatId::Curiosity, -0.5));
        let _ = bonus.push((StatId::Fulfillment, 0.2));
        if ctx.scene_plant_health > 0 {
            let _ = bonus.push((StatId::Serenity, ctx.scene_plant_health as f32 * 0.1));
        }
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
