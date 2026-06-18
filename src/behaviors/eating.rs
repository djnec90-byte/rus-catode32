use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorState},
    context::GameContext,
};

const DURATION: f32 = 3.0;
const BONUS: f32 = 25.0;
const TRIGGER_THRESHOLD: f32 = 40.0;

pub struct EatingBehavior {
    timer: f32,
}

impl EatingBehavior {
    pub fn new() -> Self {
        Self { timer: 0.0 }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.fullness < TRIGGER_THRESHOLD
    }

    pub fn priority(_ctx: &GameContext) -> u32 {
        5
    }
}

impl Behavior for EatingBehavior {
    fn name(&self) -> &'static str {
        "Eating"
    }

    fn progress(&self) -> f32 {
        (self.timer / DURATION).clamp(0.0, 1.0)
    }

    fn pose(&self) -> PoseId {
        // TODO: Python uses a phased sequence: standing.side.happy (approach) →
        // leaning_forward.side.eating (eat) → leaning_forward.side.neutral (chew),
        // with REJECTION_POSES if the cat dislikes the food.
        PoseId::LeaningForwardSideEating
    }

    fn update(&mut self, _ctx: &mut GameContext, dt: f32) -> BehaviorState {
        self.timer += dt;
        if self.timer >= DURATION {
            BehaviorState::Completed
        } else {
            BehaviorState::Running
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext) {
        ctx.fullness = (ctx.fullness + BONUS).clamp(0.0, 100.0);
    }
}
