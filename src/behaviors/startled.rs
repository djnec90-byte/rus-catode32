use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
};

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
    bob: f32,
}

impl StartledBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Startled,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 3.0,
            pose_id: PoseId::SittingForwardShocked,
            bob: 0.0,
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
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        self.bob += dt;
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
        let mut bonus: heapless::Vec<(StatId, f32), 4> = heapless::Vec::new();
        let _ = bonus.push((StatId::Comfort, -2.5));
        let _ = bonus.push((StatId::Energy, -1.0));
        let _ = bonus.push((StatId::Curiosity, 0.3));
        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, _: bool) {
        if self.phase != Phase::Startled {
            return;
        }
        // Rising exclaim above head.
        let rise = ((self.bob * 14.0) as i32).min(14);
        renderer.draw_text(
            "!",
            Point::new(char_screen.x - 2, char_screen.y - 14 - rise),
        );
    }
}
