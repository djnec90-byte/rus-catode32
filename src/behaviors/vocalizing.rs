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
    time_system::Weather,
    ui::bubble::{self, BubbleIcon},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    WindingUp,
    Vocalizing,
    Settling,
}

pub struct VocalizingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
}

impl VocalizingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::WindingUp,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 6.0,
            pose_id: PoseId::SittingForwardNeutral,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        // Vacation overstay always allows a vocalize so the pet can ask to go home.
        if ctx.wants_to_go_home {
            return true;
        }
        const NEED: f32 = 60.0;
        let happy = ctx.energy > 35.0 && ctx.playfulness > 40.0;
        let needs_unmet = ctx.fullness < NEED
            || ctx.comfort < NEED
            || ctx.fulfillment < NEED
            || ctx.affection < NEED
            || ctx.sociability < NEED;
        if common::is_outdoor(ctx.last_main_scene)
            && matches!(
                ctx.weather,
                Weather::Rain | Weather::Storm | Weather::Snow
            )
        {
            return true;
        }
        happy || needs_unmet
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        // Wants-to-go-home wins most selection rounds (Python parity).
        if ctx.wants_to_go_home {
            return rand::rand_range_f32(rng, 2.0, 8.0).max(0.0) as u32;
        }
        // Outdoor weather complaint.
        if common::is_outdoor(ctx.last_main_scene) {
            let weather_bad = matches!(
                ctx.weather,
                Weather::Rain | Weather::Storm | Weather::Snow
            );
            let temp_complaint = ctx.temperature < 2.0 || ctx.temperature > 30.0;
            if (weather_bad || temp_complaint)
                && ctx
                    .recent_index(BehaviorId::Vocalizing)
                    .is_none()
            {
                let mut urgency = 1.0_f32;
                if ctx.weather == Weather::Storm {
                    urgency = 1.5;
                }
                if ctx.temperature < -1.0 || ctx.temperature > 33.0 {
                    urgency = urgency.max(2.0);
                }
                return (rand::rand_range_f32(rng, 12.0, 22.0) / urgency).max(0.0) as u32;
            }
        }
        // Outdoor chatty.
        if common::is_outdoor(ctx.last_main_scene)
            && ctx.recent_index(BehaviorId::Vocalizing).is_none()
        {
            return rand::rand_range_f32(rng, 5.0, 15.0).max(0.0) as u32;
        }

        const NEED: f32 = 40.0;
        let urgency = (NEED - ctx.fullness)
            .max(NEED - ctx.sociability)
            .max(NEED - ctx.affection)
            .max(NEED - ctx.comfort)
            .max(NEED - ctx.playfulness)
            .max(0.0);
        if urgency > 0.0 {
            return (65.0 - urgency * 3.0).max(5.0) as u32;
        }
        let hi = ((200.0 - ctx.energy - ctx.playfulness) * 0.5).max(25.0);
        rand::rand_range_f32(rng, 25.0, hi).max(0.0) as u32
    }
}

impl Behavior for VocalizingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Vocalizing
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::WindingUp;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 5.0, 9.0);
        self.pose_id = PoseId::SittingForwardNeutral;

        // Hint icon for the speech bubble. Mirrors Python: vacation overstay →
        // home, low-fullness → meal, weather complaint → sun, low-affection → heart.
        ctx.pending_popup_icon = if ctx.wants_to_go_home {
            Some("home")
        } else if ctx.fullness < 30.0 {
            Some("hunger")
        } else if matches!(ctx.weather, Weather::Rain | Weather::Storm | Weather::Snow)
            && common::is_outdoor(ctx.last_main_scene)
        {
            Some("wet")
        } else if ctx.affection < 40.0 {
            Some("lonely")
        } else {
            Some("exclaim")
        };
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        match self.phase {
            Phase::WindingUp if self.phase_timer >= 1.0 => {
                self.phase = Phase::Vocalizing;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::YellingForwardLiftAndYell;
            }
            Phase::Vocalizing if self.phase_timer >= self.total - 2.0 => {
                self.phase = Phase::Settling;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardAloof;
            }
            Phase::Settling if self.phase_timer >= 1.5 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        if rand::rand_f32(&mut rng) < 0.2 {
            Some(NextBehavior::Zoomies)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus = [
            (StatId::Energy, -0.75 * progress),
            (StatId::Comfort, -0.3 * progress),
            (StatId::Serenity, -0.015 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        if self.phase != Phase::Vocalizing {
            return;
        }
        let icon = ctx
            .pending_popup_icon
            .and_then(BubbleIcon::from_name)
            .unwrap_or(BubbleIcon::Exclaim);
        bubble::draw_above_char(
            renderer,
            icon,
            char_screen.x,
            char_screen.y,
            self.progress(),
            mirror_h,
        );
    }
}

/// Outdoor scene helper kept here so the SceneId import isn't unused.
#[allow(dead_code)]
fn _scene_outdoor(s: SceneId) -> bool {
    common::is_outdoor(s)
}
