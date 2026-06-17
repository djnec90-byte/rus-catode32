use crate::{
    behavior::{Behavior, BehaviorState},
    context::GameContext,
};

const DURATION: f32 = 4.0;
const BONUS: f32 = 25.0;
const TRIGGER_THRESHOLD: f32 = 40.0;

pub struct SleepingBehavior {
    timer: f32,
}

impl SleepingBehavior {
    pub fn new() -> Self {
        Self { timer: 0.0 }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.energy < TRIGGER_THRESHOLD
    }

    pub fn priority(_ctx: &GameContext) -> u32 {
        10
    }
}

impl Behavior for SleepingBehavior {
    fn name(&self) -> &'static str {
        "Sleeping"
    }

    fn progress(&self) -> f32 {
        (self.timer / DURATION).clamp(0.0, 1.0)
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
        ctx.energy = (ctx.energy + BONUS).clamp(0.0, 100.0);
    }
}
