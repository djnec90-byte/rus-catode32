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
    Starting,
    Pacing,
    Stopping,
}

pub struct PacingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    dir: i32,
    speed: f32,
    walker_accum: f32,
    flip_in: f32,
}

impl PacingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Starting,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 12.0,
            pose_id: PoseId::WalkingSideDetermined,
            dir: 1,
            speed: 14.0,
            walker_accum: 0.0,
            flip_in: 3.0,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.comfort < 70.0 && ctx.serenity < 65.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let worst = ctx.comfort.min(ctx.serenity);
        let mut base = rand::rand_range_f32(rng, 10.0, (100.0 - (100.0 - worst) * 0.8).max(10.0));
        if !ctx.in_familiar_location {
            base *= 0.8;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for PacingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Pacing
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Starting;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 10.0, 18.0);
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.speed = rand::rand_range_f32(&mut ctx.rng, 12.0, 18.0);
        self.flip_in = rand::rand_range_f32(&mut ctx.rng, 2.0, 4.0);
        self.pose_id = PoseId::WalkingSideDetermined;
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
            Phase::Starting if self.phase_timer >= 0.5 => {
                self.phase = Phase::Pacing;
                self.phase_timer = 0.0;
            }
            Phase::Pacing => {
                let bounced = common::step_walker(
                    character,
                    ctx,
                    self.dir,
                    self.speed,
                    dt,
                    &mut self.walker_accum,
                );
                self.flip_in -= dt;
                if bounced || self.flip_in <= 0.0 {
                    self.dir = -self.dir;
                    self.flip_in = rand::rand_range_f32(&mut ctx.rng, 2.0, 4.0);
                }
                if self.phase_timer >= self.total - 1.0 {
                    self.phase = Phase::Stopping;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSideAnnoyed;
                }
            }
            Phase::Stopping if self.phase_timer >= 1.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        let r = rand::rand_f32(&mut rng);
        if ctx.affection < 40.0 && r < 0.25 {
            Some(NextBehavior::Vocalizing)
        } else if ctx.fulfillment < 40.0 && r < 0.45 {
            Some(NextBehavior::Sulking)
        } else if ctx.mischievousness > 50.0 && r < 0.65 {
            Some(NextBehavior::Mischief)
        } else if ctx.courage < 40.0 && r < 0.8 {
            Some(NextBehavior::Hiding)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 4> = heapless::Vec::new();
        let _ = bonus.push((StatId::Comfort, -2.0));
        let _ = bonus.push((StatId::Energy, -1.5));
        let _ = bonus.push((StatId::Fitness, 0.25));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
