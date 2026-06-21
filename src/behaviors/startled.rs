use embedded_graphics::prelude::Point;

use crate::{
    assets::{
        character::PoseId,
        icons::{EXCLAIM, EXCLAIM_H, EXCLAIM_W},
    },
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::{Renderer, SpriteOpts},
};

const EXCLAIM_RISE_DURATION: f32 = 1.5;
const EXCLAIM_RISE_AMOUNT: i32 = 15;
const EXCLAIM_WOBBLE_INTERVAL: f32 = 0.2;
const EXCLAIM_ANGLES: [f32; 3] = [-5.0, 0.0, 5.0];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Startled,
    Recovering,
}

pub struct StartledBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    wobble_timer: f32,
    wobble_angle: f32,
}

impl StartledBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Startled,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 3.0,
            pose_id: PoseId::SittingForwardShocked,
            wobble_timer: 0.0,
            wobble_angle: 0.0,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        let p = 0.45 * (1.0 - ctx.courage / 100.0);
        let mut rng = ctx.rng;
        rand::rand_f32(&mut rng) < p
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let mut base = rand::rand_range_f32(rng, 20.0, (ctx.courage * 1.2).max(20.0));
        if !ctx.in_familiar_location {
            base *= 0.75;
        }
        base.max(0.0) as u32
    }
}

impl Behavior for StartledBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Startled
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Startled;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 2.5, 4.0);
        self.pose_id = PoseId::SittingForwardShocked;
        self.wobble_timer = 0.0;
        self.wobble_angle = 0.0;
    }

    fn update(&mut self, ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        if self.phase == Phase::Startled {
            self.wobble_timer += dt;
            if self.wobble_timer >= EXCLAIM_WOBBLE_INTERVAL {
                self.wobble_timer -= EXCLAIM_WOBBLE_INTERVAL;
                let idx = rand::rand_range_u32(&mut ctx.rng, 0, 2) as usize;
                self.wobble_angle = EXCLAIM_ANGLES[idx];
            }
        }
        match self.phase {
            Phase::Startled if self.phase_timer >= self.total - 1.0 => {
                self.phase = Phase::Recovering;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingSideLookingDown;
            }
            Phase::Recovering if self.phase_timer >= 1.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, ctx: &GameContext) -> Option<NextBehavior> {
        let mut rng = ctx.rng;
        let p = (ctx.curiosity + ctx.courage) / 200.0;
        if rand::rand_f32(&mut rng) < p {
            Some(NextBehavior::Investigating)
        } else {
            None
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus = [
            (StatId::Energy, -1.5 * progress),
            (StatId::Comfort, -5.0 * progress),
            (StatId::Curiosity, 1.0 * progress),
            (StatId::Courage, -0.01 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        if self.phase != Phase::Startled {
            return;
        }
        if self.phase_timer > EXCLAIM_RISE_DURATION {
            return;
        }
        let rise_t = (self.phase_timer / EXCLAIM_RISE_DURATION).min(1.0);
        let rise_offset = (rise_t * EXCLAIM_RISE_AMOUNT as f32) as i32;
        let exclaim_y = char_screen.y - 40 - rise_offset;
        let exclaim_x = if mirror_h {
            char_screen.x + 16
        } else {
            char_screen.x - EXCLAIM_W as i32 - 16
        };
        renderer.draw_sprite_raw_rotated(
            EXCLAIM,
            EXCLAIM_W,
            EXCLAIM_H,
            Point::new(exclaim_x, exclaim_y),
            self.wobble_angle,
            SpriteOpts::default(),
        );
    }
}
