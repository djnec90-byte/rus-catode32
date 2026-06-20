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

const NAP_POSES: &[PoseId] = &[
    PoseId::LayingSideContent,
    PoseId::LayingSideBliss,
    PoseId::LayingSideNeutral,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Settling,
    Napping,
    Waking,
}

pub struct NappingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    nap_pose: PoseId,
    z_timer: f32,
}

impl NappingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Settling,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 20.0,
            pose_id: PoseId::LayingSideNeutral,
            nap_pose: PoseId::LayingSideContent,
            z_timer: 0.0,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        if ctx.sickness >= 8.0 {
            return true;
        }
        let threshold = if ctx.sickness >= 5.0 {
            97.0
        } else if ctx.sickness >= 2.0 {
            85.0
        } else {
            let mut t = if ctx.time_hours >= 21 || ctx.time_hours < 6 {
                85.0
            } else {
                60.0
            };
            if ctx.last_main_scene == SceneId::Bedroom {
                t += 20.0;
            }
            t
        };
        ctx.energy < threshold
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let lo = ctx.energy * 0.3;
        let hi = (ctx.energy * 2.5).max(ctx.energy * 0.5);
        let mut base = rand::rand_range_f32(rng, lo, hi);
        if ctx.time_hours >= 19 || ctx.time_hours < 6 {
            base *= 0.5;
        }
        if ctx.last_main_scene == SceneId::Bedroom {
            base *= 0.55;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for NappingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Napping
    }

    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }

    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _character: &mut Character) {
        self.phase = Phase::Settling;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 20.0, 40.0);
        self.nap_pose = common::pick_pose(&mut ctx.rng, NAP_POSES);
        self.pose_id = PoseId::LayingSideNeutral;
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
            Phase::Settling if self.phase_timer >= 2.5 => {
                self.phase = Phase::Napping;
                self.phase_timer = 0.0;
                self.pose_id = self.nap_pose;
            }
            Phase::Napping if self.phase_timer >= self.total - 6.0 => {
                self.phase = Phase::Waking;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::LayingSideNeutral;
                ctx.pending_wake_greeting = true;
            }
            Phase::Waking if self.phase_timer >= 2.5 => {
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
        let _ = bonus.push((StatId::Energy, 18.0));
        let _ = bonus.push((StatId::Comfort, 6.0));
        let _ = bonus.push((StatId::Focus, 6.0));
        let _ = bonus.push((StatId::Serenity, 0.6));
        let _ = bonus.push((StatId::Fullness, -3.0));
        let ph = ctx.scene_plant_health as f32;
        if ph != 0.0 {
            let _ = bonus.push((StatId::Serenity, ph * 0.1));
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
        if self.phase != Phase::Napping {
            return;
        }
        let dy = ((self.z_timer * 4.0) as i32) % 8;
        renderer.draw_text(
            "z",
            Point::new(char_screen.x - 6, char_screen.y - 12 - dy),
        );
    }

    fn mark_almost_done(&mut self) {
        self.phase = Phase::Waking;
        self.phase_timer = 0.0;
        self.elapsed = self.total - 2.0;
    }
}
