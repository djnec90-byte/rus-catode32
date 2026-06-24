use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState},
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

pub struct MeanderingBehavior {
    phase: Phase,
    phase_timer: f32,
    pose_id: PoseId,
    start_duration: f32,
    pace_duration: f32,
    stop_duration: f32,
    pace_speed: f32,
    dir: i32,
    dir_change_timer: f32,
    dir_change_interval: f32,
    walker_accum: f32,
}

impl MeanderingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Starting,
            phase_timer: 0.0,
            pose_id: PoseId::SittingSideNeutral,
            start_duration: 1.0,
            pace_duration: 20.0,
            stop_duration: 1.0,
            pace_speed: 6.0,
            dir: 1,
            dir_change_timer: 0.0,
            dir_change_interval: 3.0,
            walker_accum: 0.0,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.energy > 20.0
    }
}

impl Behavior for MeanderingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Meandering
    }
    fn progress(&self) -> f32 {
        match self.phase {
            Phase::Pacing => (self.phase_timer / self.pace_duration).clamp(0.0, 1.0),
            Phase::Starting => 0.0,
            Phase::Stopping => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Starting;
        self.phase_timer = 0.0;
        self.start_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 5.0);
        self.pace_duration = rand::rand_range_u32(&mut ctx.rng, 20, 60) as f32;
        self.stop_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 5.0);
        self.pace_speed = rand::rand_range_u32(&mut ctx.rng, 6, 9) as f32;
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.dir_change_timer = 0.0;
        self.dir_change_interval = rand::rand_range_f32(&mut ctx.rng, 3.0, 20.0);
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
                // Walk with a 20px margin off the scene edges. Bouncing off
                // either edge forces direction = +/-1 (not just inversion) and
                // re-rolls dir_change_interval.
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
                        rand::rand_range_f32(&mut ctx.rng, 3.0, 20.0);
                    character.mirror_h = true;
                } else if character.pos.x >= x_max {
                    character.pos.x = x_max;
                    self.dir = -1;
                    self.dir_change_timer = 0.0;
                    self.dir_change_interval =
                        rand::rand_range_f32(&mut ctx.rng, 3.0, 20.0);
                    character.mirror_h = false;
                }

                self.dir_change_timer += dt;
                if self.dir_change_timer >= self.dir_change_interval {
                    self.dir = -self.dir;
                    self.dir_change_timer = 0.0;
                    self.dir_change_interval =
                        rand::rand_range_f32(&mut ctx.rng, 3.0, 20.0);
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

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 10> = heapless::Vec::new();
        common::bonus_add(&mut bonus, StatId::Energy, -0.35);
        common::bonus_add(&mut bonus, StatId::Fullness, -0.25);
        common::bonus_add(&mut bonus, StatId::Playfulness, -0.15);
        common::bonus_add(&mut bonus, StatId::Comfort, -0.6);
        common::bonus_add(&mut bonus, StatId::Intelligence, -0.0015);
        common::bonus_add(&mut bonus, StatId::Fitness, 0.01);

        let ph = ctx.scene_plant_health as f32;
        if ph != 0.0 {
            common::bonus_add(&mut bonus, StatId::Comfort, ph * 0.1);
            common::bonus_add(&mut bonus, StatId::Serenity, ph * 0.1);
        }
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
