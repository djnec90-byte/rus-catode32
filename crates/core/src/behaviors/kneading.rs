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
    Kneading,
    Settling,
}

pub struct KneadingBehavior {
    phase: Phase,
    phase_timer: f32,
    knead_duration: f32,
    settle_duration: f32,
    pose_id: PoseId,
}

impl KneadingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Kneading,
            phase_timer: 0.0,
            knead_duration: 25.0,
            settle_duration: 2.0,
            pose_id: PoseId::KneadingSideNeutral,
        }
    }
}

impl Behavior for KneadingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Kneading
    }
    fn progress(&self) -> f32 {
        match self.phase {
            Phase::Kneading => (self.phase_timer / self.knead_duration).clamp(0.0, 1.0),
            Phase::Settling => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Kneading;
        self.phase_timer = 0.0;
        self.knead_duration = rand::rand_range_f32(&mut ctx.rng, 10.0, 45.0);
        self.settle_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 4.0);
        self.pose_id = PoseId::KneadingSideNeutral;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::Kneading if self.phase_timer >= self.knead_duration => {
                self.phase = Phase::Settling;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LeaningForwardSideNeutral;
            }
            Phase::Settling if self.phase_timer >= self.settle_duration => {
                return BehaviorState::Completed;
            }
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
        let mut bonus: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
        common::bonus_add(&mut bonus, StatId::Comfort, 2.0);
        common::bonus_add(&mut bonus, StatId::Focus, -0.25);
        common::bonus_add(&mut bonus, StatId::Cleanliness, -0.1);
        common::bonus_add(&mut bonus, StatId::Serenity, 0.05);

        let ff = common::fed_factor(ctx);
        if ff > 0.0 {
            common::bonus_add(&mut bonus, StatId::Comfort, 1.0 * ff);
            common::bonus_add(&mut bonus, StatId::Serenity, 0.1 * ff);
        }

        // apply_location_bonus (does NOT call super, so no fav_weather)
        if ctx.in_familiar_location {
            common::bonus_add(&mut bonus, StatId::Comfort, 1.0);
            common::bonus_add(&mut bonus, StatId::Serenity, 0.15);
        } else {
            common::bonus_scale(&mut bonus, StatId::Comfort, 0.85);
        }

        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
