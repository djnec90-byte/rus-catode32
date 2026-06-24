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
    windup_duration: f32,
    zoom_duration: f32,
    collapse_duration: f32,
    pose_id: PoseId,
    dir: i32,
    speed: f32,
    dir_change_timer: f32,
    dir_change_interval: f32,
    walker_accum: f32,
}

impl ZoomiesBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::WindingUp,
            phase_timer: 0.0,
            windup_duration: 3.0,
            zoom_duration: 30.0,
            collapse_duration: 3.0,
            pose_id: PoseId::LeaningForwardSideCrazy,
            dir: 1,
            speed: 50.0,
            dir_change_timer: 0.0,
            dir_change_interval: 1.5,
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
        match self.phase {
            Phase::WindingUp => 0.0,
            Phase::Zooming => (self.phase_timer / self.zoom_duration).clamp(0.0, 1.0),
            Phase::Collapsing => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::WindingUp;
        self.phase_timer = 0.0;
        self.windup_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 5.0);
        // Python __init__ sets uniform(15, 50) but start() overrides with randint(20, 45).
        self.zoom_duration = rand::rand_range_u32(&mut ctx.rng, 20, 45) as f32;
        self.collapse_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 5.0);
        self.speed = rand::rand_range_u32(&mut ctx.rng, 40, 60) as f32;
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.dir_change_timer = 0.0;
        self.dir_change_interval = rand::rand_range_f32(&mut ctx.rng, 1.0, 3.0);
        self.walker_accum = 0.0;
        self.pose_id = PoseId::LeaningForwardSideCrazy;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::WindingUp if self.phase_timer >= self.windup_duration => {
                self.phase = Phase::Zooming;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::RunningSideAngry;
            }
            Phase::Zooming => {
                // Walk with a 20px margin off the scene edges. Bouncing off
                // either edge forces dir = +/-1 (not just inversion) and
                // re-rolls dir_change_interval.
                let x_min = ctx.scene_x_min + 20;
                let x_max = ctx.scene_x_max - 20;
                self.walker_accum += self.speed * dt;
                let whole = self.walker_accum as i32;
                if whole != 0 {
                    self.walker_accum -= whole as f32;
                    character.pos.x += whole * self.dir.signum();
                }
                character.mirror_h = self.dir > 0;

                if character.pos.x <= x_min {
                    character.pos.x = x_min;
                    self.dir = 1;
                    self.dir_change_timer = 0.0;
                    self.dir_change_interval = rand::rand_range_f32(&mut ctx.rng, 1.0, 3.0);
                    character.mirror_h = true;
                } else if character.pos.x >= x_max {
                    character.pos.x = x_max;
                    self.dir = -1;
                    self.dir_change_timer = 0.0;
                    self.dir_change_interval = rand::rand_range_f32(&mut ctx.rng, 1.0, 3.0);
                    character.mirror_h = false;
                }

                self.dir_change_timer += dt;
                if self.dir_change_timer >= self.dir_change_interval {
                    self.dir = -self.dir;
                    self.dir_change_timer = 0.0;
                    self.dir_change_interval = rand::rand_range_f32(&mut ctx.rng, 1.0, 3.0);
                    character.mirror_h = self.dir > 0;
                }

                if self.phase_timer >= self.zoom_duration {
                    self.phase = Phase::Collapsing;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SleepingSideSploot;
                }
            }
            Phase::Collapsing if self.phase_timer >= self.collapse_duration => {
                return BehaviorState::Completed;
            }
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
