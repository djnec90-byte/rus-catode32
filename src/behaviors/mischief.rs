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
    Plotting,
    Mischief,
    Satisfied,
}

pub struct MischiefBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    dir: i32,
    speed: f32,
    walker_accum: f32,
    sub_t: f32,
    knead_phase: bool,
}

impl MischiefBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Plotting,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 12.0,
            pose_id: PoseId::SittingSideAloof,
            dir: 1,
            speed: 35.0,
            walker_accum: 0.0,
            sub_t: 0.0,
            knead_phase: false,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.mischievousness > 25.0
            && ctx.maturity < 55.0
            && ctx.playfulness > 50.0
            && ctx.energy > 40.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let hi = ((200.0 - ctx.mischievousness - ctx.playfulness) * 0.5).max(20.0);
        rand::rand_range_f32(rng, 20.0, hi).max(0.0) as u32
    }
}

impl Behavior for MischiefBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Mischief
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Plotting;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 10.0, 18.0);
        self.dir = if rand::rand_bool(&mut ctx.rng, 0.5) { 1 } else { -1 };
        self.speed = rand::rand_range_f32(&mut ctx.rng, 30.0, 45.0);
        self.pose_id = PoseId::SittingSideAloof;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        self.sub_t += dt;
        match self.phase {
            Phase::Plotting if self.phase_timer >= 1.5 => {
                self.phase = Phase::Mischief;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::RunningSideAngry;
            }
            Phase::Mischief => {
                if !self.knead_phase {
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
                }
                if self.sub_t > 2.0 {
                    self.sub_t = 0.0;
                    self.knead_phase = !self.knead_phase;
                    self.pose_id = if self.knead_phase {
                        PoseId::KneadingSideAngry
                    } else {
                        PoseId::RunningSideAngry
                    };
                }
                if self.phase_timer >= self.total - 1.0 {
                    self.phase = Phase::Satisfied;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSillySideAloof;
                }
            }
            Phase::Satisfied if self.phase_timer >= 1.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        if ctx.energy < 25.0 {
            Some(NextBehavior::Hiding)
        } else {
            Some(NextBehavior::Pacing)
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 5> = heapless::Vec::new();
        let _ = bonus.push((StatId::Energy, -3.5));
        let _ = bonus.push((StatId::Playfulness, -3.0));
        let _ = bonus.push((StatId::Mischievousness, 0.5));
        let _ = bonus.push((StatId::Loyalty, -0.4));
        let _ = bonus.push((StatId::Cleanliness, -1.0));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
