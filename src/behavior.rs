use crate::{behaviors::ActiveBehavior, context::GameContext};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviorId {
    Idle,
    Eating,
    Sleeping,
}

pub enum BehaviorState {
    Running,
    Completed,
}

pub trait Behavior {
    fn name(&self) -> &'static str;
    fn progress(&self) -> f32;

    fn enter(&mut self, _ctx: &mut GameContext) {}
    fn update(&mut self, ctx: &mut GameContext, dt: f32) -> BehaviorState;
    fn exit(&mut self, _ctx: &mut GameContext, _completed: bool) {}
    fn apply_completion_bonus(&self, _ctx: &mut GameContext) {}
    fn next(&self, _ctx: &GameContext) -> Option<BehaviorId> {
        None
    }
}

pub struct BehaviorManager {
    current: ActiveBehavior,
}

impl BehaviorManager {
    pub fn new() -> Self {
        Self {
            current: ActiveBehavior::from_id(BehaviorId::Idle),
        }
    }

    pub fn start(&mut self, ctx: &mut GameContext) {
        self.current.as_behavior_mut().enter(ctx);
    }

    pub fn update(&mut self, ctx: &mut GameContext, dt: f32) {
        match self.current.as_behavior_mut().update(ctx, dt) {
            BehaviorState::Running => {}
            BehaviorState::Completed => self.advance(ctx, true),
        }
    }

    pub fn skip(&mut self, ctx: &mut GameContext) {
        self.advance(ctx, false);
    }

    fn advance(&mut self, ctx: &mut GameContext, completed: bool) {
        self.current.as_behavior_mut().exit(ctx, completed);
        if completed {
            self.current.as_behavior().apply_completion_bonus(ctx);
        }
        let next_id = if completed {
            self.current
                .as_behavior()
                .next(ctx)
                .unwrap_or_else(|| crate::behaviors::select(ctx))
        } else {
            crate::behaviors::select(ctx)
        };
        self.current = ActiveBehavior::from_id(next_id);
        self.current.as_behavior_mut().enter(ctx);
    }

    pub fn current_name(&self) -> &'static str {
        self.current.as_behavior().name()
    }

    pub fn current_progress(&self) -> f32 {
        self.current.as_behavior().progress()
    }
}
