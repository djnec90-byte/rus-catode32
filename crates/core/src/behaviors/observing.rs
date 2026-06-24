use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    behaviors::common,
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
    notice_duration: f32,
    watch_duration: f32,
    lose_interest_duration: f32,
    pose_id: PoseId,
}

impl ObservingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Noticing,
            phase_timer: 0.0,
            notice_duration: 4.0,
            watch_duration: 22.0,
            lose_interest_duration: 4.0,
            pose_id: PoseId::SittingSideLookingDown,
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
        match self.phase {
            Phase::Noticing => 0.0,
            Phase::Watching => (self.phase_timer / self.watch_duration).clamp(0.0, 1.0),
            Phase::LosingInterest => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Noticing;
        self.phase_timer = 0.0;
        self.notice_duration = rand::rand_range_f32(&mut ctx.rng, 2.0, 6.0);
        self.watch_duration = rand::rand_range_f32(&mut ctx.rng, 15.0, 30.0);
        self.lose_interest_duration = rand::rand_range_f32(&mut ctx.rng, 2.0, 6.0);
        self.pose_id = PoseId::SittingSideLookingDown;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::Noticing if self.phase_timer >= self.notice_duration => {
                self.phase = Phase::Watching;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LeaningForwardSideNeutral;
            }
            Phase::Watching if self.phase_timer >= self.watch_duration => {
                self.phase = Phase::LosingInterest;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSillySideNeutral;
            }
            Phase::LosingInterest if self.phase_timer >= self.lose_interest_duration => {
                return BehaviorState::Completed;
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        if ctx.playfulness > 60.0 && rand::rand_f32(&mut rng) < 0.4 {
            return Some(NextBehavior::Chattering);
        }
        if ctx.focus > 55.0 && rand::rand_f32(&mut rng) < 0.3 {
            return Some(NextBehavior::Investigating);
        }
        None
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
        common::bonus_add(&mut bonus, StatId::Focus, -0.4);
        common::bonus_add(&mut bonus, StatId::Playfulness, -0.15);
        common::bonus_add(&mut bonus, StatId::Curiosity, -0.05);
        common::bonus_add(&mut bonus, StatId::Maturity, 0.025);
        common::bonus_add(&mut bonus, StatId::Serenity, -0.02);
        let ph = ctx.scene_plant_health as f32;
        if ph != 0.0 {
            common::bonus_add(&mut bonus, StatId::Serenity, ph * 0.1);
            common::bonus_add(&mut bonus, StatId::Fulfillment, ph * 0.05);
            common::bonus_add(&mut bonus, StatId::Curiosity, ph * 0.05);
        }
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
