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
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    dir: i32,
    speed: f32,
    walker_accum: f32,
}

impl MeanderingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Starting,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 15.0,
            pose_id: PoseId::WalkingSideNeutral,
            dir: 1,
            speed: 6.0,
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
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Starting;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 12.0, 22.0);
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.speed = rand::rand_range_f32(&mut ctx.rng, 5.0, 8.0);
        self.pose_id = PoseId::WalkingSideNeutral;
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
            Phase::Starting if self.phase_timer >= 1.0 => {
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
                if bounced || rand::rand_f32(&mut ctx.rng) < dt * 0.15 {
                    self.dir = -self.dir;
                }
                if self.phase_timer >= self.total - 1.5 {
                    self.phase = Phase::Stopping;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSideNeutral;
                }
            }
            Phase::Stopping if self.phase_timer >= 1.5 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 6> = heapless::Vec::new();
        let _ = bonus.push((StatId::Energy, -1.0));
        let _ = bonus.push((StatId::Comfort, -0.3));
        let _ = bonus.push((StatId::Curiosity, 0.2));
        let _ = bonus.push((StatId::Fitness, 0.1));
        let ph = ctx.scene_plant_health as f32;
        if ph != 0.0 {
            let _ = bonus.push((StatId::Comfort, ph * 0.1));
            let _ = bonus.push((StatId::Serenity, ph * 0.1));
        }
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
