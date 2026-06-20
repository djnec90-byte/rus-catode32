use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    behaviors::common,
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
    scene::SceneId,
};

const SLEEP_POSES: &[PoseId] = &[
    PoseId::SleepingSideSploot,
    PoseId::SleepingSideModest,
    PoseId::SleepingSideCrossed,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Considering,
    Settling,
    Sleeping,
    Waking,
}

pub struct SleepingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    sleep_pose: PoseId,
    z_timer: f32,
}

impl SleepingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Considering,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 50.0,
            pose_id: PoseId::SittingForwardSleepy,
            sleep_pose: PoseId::SleepingSideModest,
            z_timer: 0.0,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        if ctx.sickness >= 8.0 {
            return true;
        }
        let threshold = if ctx.sickness >= 5.0 {
            95.0
        } else if ctx.sickness >= 2.0 {
            75.0
        } else {
            let mut t = if ctx.time_hours >= 21 || ctx.time_hours < 6 {
                70.0
            } else {
                40.0
            };
            if ctx.last_main_scene == SceneId::Bedroom {
                t += 20.0;
            }
            t
        };
        ctx.energy < threshold
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let lo = ctx.energy * 0.25;
        let hi = (ctx.energy * 2.0).max(lo);
        let mut base = rand::rand_range_f32(rng, lo, hi);
        if ctx.time_hours >= 19 || ctx.time_hours < 6 {
            base *= 0.4;
        }
        if ctx.last_main_scene == SceneId::Bedroom {
            base *= 0.55;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for SleepingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Sleeping
    }

    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }

    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _character: &mut Character) {
        self.phase = Phase::Considering;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 45.0, 90.0);
        self.sleep_pose = common::pick_pose(&mut ctx.rng, SLEEP_POSES);
        self.pose_id = PoseId::SittingForwardSleepy;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        _character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        self.z_timer += dt;
        match self.phase {
            Phase::Considering if self.phase_timer >= 3.0 => {
                self.phase = Phase::Settling;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideNeutral;
            }
            Phase::Settling if self.phase_timer >= 3.0 => {
                self.phase = Phase::Sleeping;
                self.phase_timer = 0.0;
                self.pose_id = self.sleep_pose;
            }
            Phase::Sleeping if self.phase_timer >= self.total - 9.0 => {
                self.phase = Phase::Waking;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideNeutral;
                ctx.pending_wake_greeting = true;
            }
            Phase::Waking if self.phase_timer >= 3.0 => {
                return BehaviorState::Completed;
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, _ctx: &GameContext) -> Option<NextBehavior> {
        Some(NextBehavior::Stretching)
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
        let _ = bonus.push((StatId::Energy, 35.0));
        let _ = bonus.push((StatId::Comfort, 10.0));
        let _ = bonus.push((StatId::Focus, 12.0));
        let _ = bonus.push((StatId::Serenity, 1.2));
        let _ = bonus.push((StatId::Fullness, -8.0));
        if ctx.in_cat_bed {
            let _ = bonus.push((StatId::Comfort, 4.0));
            let _ = bonus.push((StatId::Serenity, 0.6));
        }
        let ph = ctx.scene_plant_health as f32;
        if ph != 0.0 {
            let _ = bonus.push((StatId::Serenity, ph * 0.15));
            let _ = bonus.push((StatId::Comfort, ph * 0.1));
        }
        for entry in bonus.iter_mut() {
            entry.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _ctx: &GameContext,
        char_screen: Point,
        _mirror_h: bool,
    ) {
        if self.phase != Phase::Sleeping {
            return;
        }
        // Four animated wavy Z's drifting up.
        let t = self.z_timer;
        for i in 0..4 {
            let phase = t + i as f32 * 0.6;
            let dx = ((phase * 2.0) as i32) % 6 - 3;
            let dy = (phase * 8.0) as i32 % 24;
            let pos = Point::new(char_screen.x - 6 + dx, char_screen.y - 16 - dy);
            renderer.draw_text("z", pos);
        }
    }

    fn mark_almost_done(&mut self) {
        // Skip directly to the waking phase so the wake greeting fires quickly.
        self.phase = Phase::Waking;
        self.phase_timer = 0.0;
        self.elapsed = self.total - 3.0;
    }
}
