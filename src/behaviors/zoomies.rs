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
    WindingUp,
    Zooming,
    Collapsing,
}

pub struct ZoomiesBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    dir: i32,
    speed: f32,
    walker_accum: f32,
}

impl ZoomiesBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::WindingUp,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 8.0,
            pose_id: PoseId::SittingForwardShocked,
            dir: 1,
            speed: 60.0,
            walker_accum: 0.0,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.energy > 40.0 && ctx.playfulness > 40.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let lo = 100.0 - ctx.playfulness * 1.5;
        let hi = ctx.playfulness * 1.5;
        let mut base = rand::rand_range_f32(rng, lo, hi.max(lo));
        if ctx.in_familiar_location {
            base *= 0.85;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for ZoomiesBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Zoomies
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::WindingUp;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 6.0, 12.0);
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.speed = rand::rand_range_f32(&mut ctx.rng, 55.0, 80.0);
        self.pose_id = PoseId::StandingSideCrazy;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::WindingUp if self.phase_timer >= 1.0 => {
                self.phase = Phase::Zooming;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::RunningSideCrazy;
            }
            Phase::Zooming => {
                let bounced = common::step_walker(
                    character,
                    ctx,
                    self.dir,
                    self.speed,
                    dt,
                    &mut self.walker_accum,
                );
                if bounced {
                    self.dir = -self.dir;
                }
                if self.phase_timer >= self.total - 1.5 {
                    self.phase = Phase::Collapsing;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::LayingSideBored;
                }
            }
            Phase::Collapsing if self.phase_timer >= 1.5 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        if rand::rand_f32(&mut rng) < 0.2 {
            Some(NextBehavior::Vocalizing)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 10> = heapless::Vec::new();
        common::bonus_add(&mut bonus, StatId::Energy, -3.5);
        common::bonus_add(&mut bonus, StatId::Fullness, -0.5);
        common::bonus_add(&mut bonus, StatId::Playfulness, -0.3);
        common::bonus_add(&mut bonus, StatId::Cleanliness, -0.3);
        common::bonus_add(&mut bonus, StatId::Maturity, -0.1);
        common::bonus_add(&mut bonus, StatId::Comfort, -0.2);
        common::bonus_add(&mut bonus, StatId::Intelligence, -0.005);
        common::bonus_add(&mut bonus, StatId::Fitness, 0.02);
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
