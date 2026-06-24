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
    start_duration: f32,
    pace_duration: f32,
    stop_duration: f32,
    pose_id: PoseId,
    dir: i32,
    pace_speed: f32,
    dir_change_timer: f32,
    dir_change_interval: f32,
    walker_accum: f32,
}

impl PacingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Starting,
            phase_timer: 0.0,
            start_duration: 2.0,
            pace_duration: 25.0,
            stop_duration: 2.0,
            pose_id: PoseId::SittingSideNeutral,
            dir: 1,
            pace_speed: 20.0,
            dir_change_timer: 0.0,
            dir_change_interval: 4.5,
            walker_accum: 0.0,
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
        match self.phase {
            Phase::Starting => 0.0,
            Phase::Pacing => (self.phase_timer / self.pace_duration).clamp(0.0, 1.0),
            Phase::Stopping => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Starting;
        self.phase_timer = 0.0;
        self.start_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 3.0);
        self.pace_duration = rand::rand_range_u32(&mut ctx.rng, 10, 45) as f32;
        self.stop_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 3.0);
        self.pace_speed = rand::rand_range_u32(&mut ctx.rng, 15, 25) as f32;
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.dir_change_timer = 0.0;
        // Python __init__ seeds dir_change_interval at uniform(2.0, 7.0); only
        // subsequent re-rolls (bounce / mid-pace flip) tighten to uniform(3.0, 6.0).
        self.dir_change_interval = rand::rand_range_f32(&mut ctx.rng, 2.0, 7.0);
        self.walker_accum = 0.0;
        self.pose_id = PoseId::SittingSideNeutral;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::Starting if self.phase_timer >= self.start_duration => {
                self.phase = Phase::Pacing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::WalkingSideNeutral;
            }
            Phase::Pacing => {
                let x_min = ctx.scene_x_min + 20;
                let x_max = ctx.scene_x_max - 20;
                self.walker_accum += self.pace_speed * dt;
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
                    self.dir_change_interval =
                        rand::rand_range_f32(&mut ctx.rng, 3.0, 6.0);
                    character.mirror_h = true;
                } else if character.pos.x >= x_max {
                    character.pos.x = x_max;
                    self.dir = -1;
                    self.dir_change_timer = 0.0;
                    self.dir_change_interval =
                        rand::rand_range_f32(&mut ctx.rng, 3.0, 6.0);
                    character.mirror_h = false;
                }

                self.dir_change_timer += dt;
                if self.dir_change_timer >= self.dir_change_interval {
                    self.dir = -self.dir;
                    self.dir_change_timer = 0.0;
                    self.dir_change_interval =
                        rand::rand_range_f32(&mut ctx.rng, 3.0, 6.0);
                    character.mirror_h = self.dir > 0;
                }

                if self.phase_timer >= self.pace_duration {
                    self.phase = Phase::Stopping;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSideAloof;
                }
            }
            Phase::Stopping if self.phase_timer >= self.stop_duration => {
                return BehaviorState::Completed;
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        // Vocalize if immature and social. Each branch uses an independent roll.
        let p_vocalize = (1.0 - ctx.maturity / 100.0) * (ctx.sociability / 100.0);
        if rand::rand_f32(&mut rng) < p_vocalize {
            return Some(NextBehavior::Vocalizing);
        }
        // Sulk if emotionally depleted (50/50 if both fulfillment and affection are low).
        if ctx.fulfillment < 40.0
            && ctx.affection < 40.0
            && rand::rand_f32(&mut rng) < 0.5
        {
            return Some(NextBehavior::Sulking);
        }
        // Act out if immature, devious, playful, and still has energy.
        if ctx.mischievousness > 30.0
            && ctx.maturity < 40.0
            && ctx.playfulness > 60.0
            && ctx.energy > 50.0
            && rand::rand_f32(&mut rng) < 0.5
        {
            return Some(NextBehavior::Mischief);
        }
        // Retreat if scared, depleted, and out of coping resources.
        if ctx.courage < 60.0
            && ctx.affection < 60.0
            && ctx.energy < 60.0
            && rand::rand_f32(&mut rng) < 0.4
        {
            return Some(NextBehavior::Hiding);
        }
        None
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 10> = heapless::Vec::new();
        common::bonus_add(&mut bonus, StatId::Cleanliness, -0.1);
        common::bonus_add(&mut bonus, StatId::Fulfillment, -0.035);
        common::bonus_add(&mut bonus, StatId::Affection, -0.02);
        common::bonus_add(&mut bonus, StatId::Comfort, -1.5);
        common::bonus_add(&mut bonus, StatId::Fitness, 0.01);
        common::bonus_add(&mut bonus, StatId::Loyalty, -0.01);
        common::bonus_add(&mut bonus, StatId::Mischievousness, 0.005);

        let hf = common::hungry_factor(ctx);
        if hf > 0.0 {
            common::bonus_add(&mut bonus, StatId::Loyalty, -0.04 * hf);
            common::bonus_add(&mut bonus, StatId::Fulfillment, -0.04 * hf);
            common::bonus_add(&mut bonus, StatId::Affection, -0.03 * hf);
            common::bonus_add(&mut bonus, StatId::Serenity, -0.5 * hf);
        }

        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
