use crate::{
    behavior::{Behavior, BehaviorState},
    context::GameContext,
};

const DURATION: f32 = 5.0;

pub struct IdleBehavior {
    timer: f32,
}

impl IdleBehavior {
    pub fn new() -> Self {
        Self { timer: 0.0 }
    }
}

impl Behavior for IdleBehavior {
    fn name(&self) -> &'static str {
        "Idle"
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
}
