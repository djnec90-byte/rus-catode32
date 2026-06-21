use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState},
    behaviors::common,
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
    ui::bubble::{self, BubbleIcon},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Walking,
    Sniffing,
    Reacting,
}

pub struct GreetingBehavior {
    phase: Phase,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    target_x: Option<i32>,
    walker_accum: f32,
}

impl GreetingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Sniffing,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 6.0,
            pose_id: PoseId::StandingSideSniffing,
            target_x: None,
            walker_accum: 0.0,
        }
    }
}

impl Behavior for GreetingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Greeting
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = if self.target_x.is_some() {
            Phase::Walking
        } else {
            Phase::Sniffing
        };
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(&mut ctx.rng, 5.0, 8.0);
        self.pose_id = PoseId::StandingSideSniffing;
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
            Phase::Walking => {
                if let Some(tx) = self.target_x {
                    let dir = if character.pos.x < tx { 1 } else { -1 };
                    common::step_walker(
                        character,
                        ctx,
                        dir,
                        14.0,
                        dt,
                        &mut self.walker_accum,
                    );
                    if common::distance_to(character, tx) <= 1 {
                        self.phase = Phase::Sniffing;
                        self.phase_timer = 0.0;
                    }
                }
            }
            Phase::Sniffing if self.phase_timer >= 2.5 => {
                self.phase = Phase::Reacting;
                self.phase_timer = 0.0;
                self.pose_id = PoseId::SittingForwardHappy;
            }
            Phase::Reacting if self.phase_timer >= 2.0 => return BehaviorState::Completed,
            _ => {}
        }
        BehaviorState::Running
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus = [
            (StatId::Sociability, 0.25 * progress),
            (StatId::Affection, 0.3 * progress),
            (StatId::Serenity, 0.1 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        if matches!(self.phase, Phase::Sniffing) {
            bubble::draw_above_char(
                renderer,
                BubbleIcon::Question,
                char_screen.x,
                char_screen.y,
                self.progress(),
                mirror_h,
            );
        }
    }
}
