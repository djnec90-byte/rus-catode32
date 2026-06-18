use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior, TrainingKind},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WarmingUp,
    Training,
    CoolingDown,
    Rejecting,
}

pub struct TrainingBehavior {
    kind: TrainingKind,
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    alt_pose: PoseId,
    rejected: bool,
    swap_t: f32,
}

impl TrainingBehavior {
    pub fn new(kind: TrainingKind) -> Self {
        Self {
            kind,
            phase: Phase::WarmingUp,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 12.0,
            pose_id: PoseId::BeggingSideArmUp,
            alt_pose: PoseId::BeggingSideArmUp2,
            rejected: false,
            swap_t: 0.0,
        }
    }
}

impl Behavior for TrainingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Training
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.rejected = ctx.energy < 25.0 || ctx.affection < 25.0;
        self.phase = if self.rejected {
            Phase::Rejecting
        } else {
            Phase::WarmingUp
        };
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 10.0, 16.0);
        let poses = [
            PoseId::BeggingSideArmUp,
            PoseId::BeggingSideArmUp2,
            PoseId::BeggingSideDemanding,
        ];
        let i = rand::rand_range_u32(&mut ctx.rng, 0, 2) as usize;
        let j = (i + 1) % 3;
        self.pose_id = if self.rejected {
            PoseId::SittingSideAnnoyed
        } else {
            poses[i]
        };
        self.alt_pose = poses[j];
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        self.swap_t += dt;
        match self.phase {
            Phase::Rejecting if self.phase_timer >= 3.0 => return BehaviorState::Completed,
            Phase::WarmingUp if self.phase_timer >= 1.5 => {
                self.phase = Phase::Training;
                self.phase_timer = 0.0;
            }
            Phase::Training => {
                if self.swap_t >= 1.5 {
                    self.swap_t = 0.0;
                    core::mem::swap(&mut self.pose_id, &mut self.alt_pose);
                }
                if self.phase_timer >= self.total - 2.0 {
                    self.phase = Phase::CoolingDown;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSideAloof;
                }
            }
            Phase::CoolingDown if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        if self.rejected {
            return Some(NextBehavior::Meandering);
        }
        let mut rng = ctx.rng;
        if rand::rand_f32(&mut rng) < 0.5 {
            // Toy variant kept as Mouse until inventory exists.
            Some(NextBehavior::Playing(crate::behavior::PlayVariant::Mouse))
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        if self.rejected {
            return;
        }
        // Stat tables per training_type. Mirrors Python training.py.
        let (a, b, c, d) = match self.kind {
            TrainingKind::Intelligence => {
                (StatId::Intelligence, 6.0, StatId::Focus, 2.0)
            }
            TrainingKind::Behavior => (StatId::Loyalty, 4.0, StatId::Maturity, 3.0),
            TrainingKind::Fitness => (StatId::Fitness, 5.0, StatId::Energy, -4.0),
            TrainingKind::Sociability => {
                (StatId::Sociability, 5.0, StatId::Affection, 2.0)
            }
        };
        let bonus = [
            (a, b * progress),
            (c, d * progress),
            (StatId::Fulfillment, 0.6 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }
}
