use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    behaviors::common,
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
};

const NEUTRAL_LOUNGE: &[PoseId] = &[
    PoseId::LayingSideNeutral,
    PoseId::LayingSideNeutral2,
    PoseId::LayingSideContent,
    PoseId::LayingSideBored,
    PoseId::LayingSideAloof,
];

const HAPPY_LOUNGE: &[PoseId] = &[
    PoseId::LayingSideHappy,
    PoseId::LayingSideContent,
    PoseId::LayingSideBliss,
    PoseId::LayingSideAloof,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Settling,
    Lounging,
    Rousing,
}

pub struct LoungingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    lounge_pose: PoseId,
}

impl LoungingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Settling,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 18.0,
            pose_id: PoseId::SittingSideNeutral,
            lounge_pose: PoseId::LayingSideContent,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        ctx.focus > 30.0 && ctx.serenity > 30.0
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let lo = ctx.serenity * 0.5;
        let hi = (ctx.serenity * 1.5).min(90.0);
        let mut base = 100.0 - rand::rand_range_f32(rng, lo, hi);
        if ctx.in_familiar_location {
            base *= 0.8;
        }
        base.max(5.0) as u32
    }
}

impl Behavior for LoungingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Lounging
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Settling;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 20.0, 45.0);
        let happy = ctx.fullness >= 50.0
            && ctx.comfort >= 50.0
            && ctx.affection >= 60.0
            && ctx.serenity >= 60.0;
        let pool = if happy { HAPPY_LOUNGE } else { NEUTRAL_LOUNGE };
        self.lounge_pose = common::pick_pose(&mut ctx.rng, pool);
        self.pose_id = PoseId::SittingSideAloof;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::Settling if self.phase_timer >= 2.0 => {
                self.phase = Phase::Lounging;
                self.phase_timer = 0.0;
                self.pose_id = self.lounge_pose;
            }
            Phase::Lounging if self.phase_timer >= self.total - 3.0 => {
                self.phase = Phase::Rousing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideNeutral;
            }
            Phase::Rousing if self.phase_timer >= 3.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        let r = rand::rand_f32(&mut rng);
        if ctx.comfort < 50.0 && r < 0.4 {
            Some(NextBehavior::Kneading)
        } else if ctx.energy < 40.0 && r < 0.7 {
            Some(NextBehavior::Napping)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
        let _ = bonus.push((StatId::Comfort, 6.0));
        let _ = bonus.push((StatId::Serenity, 1.5));
        let _ = bonus.push((StatId::Fulfillment, 0.4));
        let _ = bonus.push((StatId::Energy, -1.5));
        let _ = bonus.push((StatId::Fullness, -0.8));
        let (c, s) = common::fav_weather_bonus(ctx);
        if c != 0.0 {
            let _ = bonus.push((StatId::Comfort, c));
        }
        if s != 0.0 {
            let _ = bonus.push((StatId::Serenity, s));
        }
        let ph = ctx.scene_plant_health as f32;
        if ph != 0.0 {
            let _ = bonus.push((StatId::Serenity, ph * 0.2));
            let _ = bonus.push((StatId::Comfort, ph * 0.15));
            let _ = bonus.push((StatId::Fulfillment, ph * 0.05));
        }
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }
}
