use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

const FIND_DURATION: f32 = 1.5;
const HIDE_DURATION: f32 = 12.0;
const EMERGE_DURATION: f32 = 1.5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    FindingSpot,
    Hiding,
    Emerging,
}

pub struct HidingBehavior {
    phase: Phase,
    phase_timer: f32,
    pose_id: PoseId,
}

impl HidingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::FindingSpot,
            phase_timer: 0.0,
            pose_id: PoseId::LayingSideBored,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.courage < 65.0 && (ctx.affection < 55.0 || ctx.energy < 55.0)
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let mut base = rand::rand_range_f32(rng, 15.0, ctx.courage.max(15.0));
        if ctx.in_familiar_location {
            base *= 1.4;
        } else {
            base *= 0.65;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for HidingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Hiding
    }
    fn progress(&self) -> f32 {
        match self.phase {
            Phase::FindingSpot => 0.0,
            Phase::Hiding => (self.phase_timer / HIDE_DURATION).clamp(0.0, 1.0),
            Phase::Emerging => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, _ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::FindingSpot;
        self.phase_timer = 0.0;
        self.pose_id = PoseId::LayingSideBored;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::FindingSpot if self.phase_timer >= FIND_DURATION => {
                self.phase = Phase::Hiding;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideContent;
            }
            Phase::Hiding if self.phase_timer >= HIDE_DURATION => {
                self.phase = Phase::Emerging;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideBored;
            }
            Phase::Emerging if self.phase_timer >= EMERGE_DURATION => {
                return BehaviorState::Completed;
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus = [
            (StatId::Comfort, 0.2 * progress),
            (StatId::Sociability, -0.25 * progress),
            (StatId::Affection, -0.1 * progress),
            (StatId::Courage, -0.005 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }
}
