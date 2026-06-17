pub mod eating;
pub mod idle;
pub mod sleeping;

use crate::{
    behavior::{Behavior, BehaviorId},
    context::GameContext,
};

use eating::EatingBehavior;
use idle::IdleBehavior;
use sleeping::SleepingBehavior;

pub enum ActiveBehavior {
    Idle(IdleBehavior),
    Eating(EatingBehavior),
    Sleeping(SleepingBehavior),
}

impl ActiveBehavior {
    pub fn from_id(id: BehaviorId) -> Self {
        match id {
            BehaviorId::Idle => ActiveBehavior::Idle(IdleBehavior::new()),
            BehaviorId::Eating => ActiveBehavior::Eating(EatingBehavior::new()),
            BehaviorId::Sleeping => ActiveBehavior::Sleeping(SleepingBehavior::new()),
        }
    }

    pub fn as_behavior_mut(&mut self) -> &mut dyn Behavior {
        match self {
            ActiveBehavior::Idle(b) => b,
            ActiveBehavior::Eating(b) => b,
            ActiveBehavior::Sleeping(b) => b,
        }
    }

    pub fn as_behavior(&self) -> &dyn Behavior {
        match self {
            ActiveBehavior::Idle(b) => b,
            ActiveBehavior::Eating(b) => b,
            ActiveBehavior::Sleeping(b) => b,
        }
    }
}

struct Candidate {
    id: BehaviorId,
    can_trigger: fn(&GameContext) -> bool,
    priority: fn(&GameContext) -> u32,
}

const CANDIDATES: &[Candidate] = &[
    Candidate {
        id: BehaviorId::Eating,
        can_trigger: EatingBehavior::can_trigger,
        priority: EatingBehavior::priority,
    },
    Candidate {
        id: BehaviorId::Sleeping,
        can_trigger: SleepingBehavior::can_trigger,
        priority: SleepingBehavior::priority,
    },
];

pub fn select(ctx: &GameContext) -> BehaviorId {
    let mut best: Option<(BehaviorId, u32)> = None;
    for c in CANDIDATES {
        if (c.can_trigger)(ctx) {
            let p = (c.priority)(ctx);
            if best.map_or(true, |(_, bp)| p < bp) {
                best = Some((c.id, p));
            }
        }
    }
    best.map(|(id, _)| id).unwrap_or(BehaviorId::Idle)
}
