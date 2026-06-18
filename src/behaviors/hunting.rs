use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, GiftKind, NextBehavior},
    behaviors::common,
    context::{FoodKind, GameContext, StatId},
    entities::character::Character,
    rand,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Stalking,
    Darting,
    Pouncing,
    Catching,
    Missing,
}

pub struct HuntingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    dir: i32,
    speed: f32,
    walker_accum: f32,
    success: bool,
}

impl HuntingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Stalking,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 12.0,
            pose_id: PoseId::StandingSideSniffing,
            dir: 1,
            speed: 40.0,
            walker_accum: 0.0,
            success: true,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        if ctx.fullness < 15.0 && ctx.energy > 20.0 {
            return true;
        }
        let outdoor = common::is_outdoor(ctx.last_main_scene);
        let threshold = if outdoor { 15.0 } else { 20.0 };
        ctx.energy > threshold && ctx.playfulness > threshold
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let hunger_pull = 100.0 - ctx.fullness;
        let play_pull = ctx.playfulness;
        let ceiling = (85.0 - play_pull * 0.5 - hunger_pull * 0.3).max(25.0);
        let floor = (25.0 + hunger_pull * 0.15 - play_pull * 0.1).max(10.0);
        let mut base = rand::rand_range_f32(rng, floor, (floor + 5.0).max(ceiling));
        if common::is_outdoor(ctx.last_main_scene) {
            base *= 0.75;
        }
        if ctx.fullness < 5.0 {
            base = base.min(15.0 + ctx.fullness * 0.5);
        }
        base.max(0.0) as u32
    }
}

impl Behavior for HuntingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Hunting
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Stalking;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 10.0, 16.0);
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.speed = rand::rand_range_f32(&mut ctx.rng, 35.0, 55.0);
        self.success = rand::rand_bool(&mut ctx.rng, 0.6);
        self.pose_id = PoseId::StandingSideSniffing;
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
            Phase::Stalking if self.phase_timer >= 2.0 => {
                self.phase = Phase::Darting;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::RunningSideNeutral;
            }
            Phase::Darting => {
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
                if self.phase_timer >= self.total - 4.0 {
                    self.phase = Phase::Pouncing;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::LeaningForwardSidePounce;
                }
            }
            Phase::Pouncing if self.phase_timer >= 1.5 => {
                if self.success {
                    self.phase = Phase::Catching;
                    self.pose_id = PoseId::SittingSideHappy;
                } else {
                    self.phase = Phase::Missing;
                    self.pose_id = PoseId::SittingSideAnnoyed;
                }
                self.phase_timer = 0.0;
            }
            Phase::Catching | Phase::Missing if self.phase_timer >= 1.5 => {
                return BehaviorState::Completed
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        if !self.success {
            return None;
        }
        // Hungry → eat the catch. Otherwise present it as a gift.
        if ctx.fullness < 40.0 {
            Some(NextBehavior::Eating(FoodKind::CaughtSnack))
        } else {
            Some(NextBehavior::GiftBringing(GiftKind::Mouse))
        }
    }

    fn exit(&mut self, ctx: &mut GameContext, completed: bool) {
        if completed && self.success {
            ctx.coins = (ctx.coins + 1).min(9999);
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus = [
            (StatId::Energy, -10.0 * progress),
            (StatId::Playfulness, -6.0 * progress),
            (StatId::Fitness, 0.7 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }
}
