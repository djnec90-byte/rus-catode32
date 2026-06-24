use embedded_graphics::prelude::Point;

use crate::{
    assets::{
        character::PoseId,
        items::{FISH1, MOUSE_TOY},
    },
    behavior::{Behavior, BehaviorId, BehaviorState, GiftKind},
    context::{GameContext, StatId},
    entities::character::Character,
    render::{Renderer, Sprite, SpriteOpts},
};

const APPROACH_DURATION: f32 = 1.5;
const PRESENT_DURATION: f32 = 8.0;
const SATISFY_DURATION: f32 = 1.5;
const GIFT_OFFSET_X: i32 = 30;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Approaching,
    Presenting,
    Satisfied,
}

pub struct GiftBringingBehavior {
    gift: GiftKind,
    phase: Phase,
    phase_timer: f32,
    gift_y_progress: f32,
    pose_id: PoseId,
}

impl GiftBringingBehavior {
    pub fn new(gift: GiftKind) -> Self {
        Self {
            gift,
            phase: Phase::Approaching,
            phase_timer: 0.0,
            gift_y_progress: 0.0,
            pose_id: PoseId::SittingSideHappy,
        }
    }

    fn gift_sprite(&self) -> &'static Sprite {
        match self.gift {
            GiftKind::Fish => &FISH1,
            GiftKind::Mouse => &MOUSE_TOY,
        }
    }
}

impl Behavior for GiftBringingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::GiftBringing
    }
    fn progress(&self) -> f32 {
        match self.phase {
            Phase::Approaching => 0.0,
            Phase::Presenting => (self.phase_timer / PRESENT_DURATION).clamp(0.0, 1.0),
            Phase::Satisfied => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, _ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Approaching;
        self.phase_timer = 0.0;
        self.gift_y_progress = 0.0;
        self.pose_id = PoseId::SittingSideHappy;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::Approaching => {
                self.gift_y_progress = (self.phase_timer / APPROACH_DURATION).min(1.0);
                if self.phase_timer >= APPROACH_DURATION {
                    self.phase = Phase::Presenting;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::YellingForwardLiftAndYell;
                }
            }
            Phase::Presenting => {
                if self.phase_timer >= PRESENT_DURATION {
                    self.phase = Phase::Satisfied;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSideHappy;
                    character.play_bursts(&mut ctx.rng, 5);
                }
            }
            Phase::Satisfied => {
                if self.phase_timer >= SATISFY_DURATION {
                    return BehaviorState::Completed;
                }
            }
        }
        BehaviorState::Running
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let bonus = [
            (StatId::Sociability, 0.5 * progress),
            (StatId::Affection, 0.25 * progress),
            (StatId::Loyalty, 0.1 * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _ctx: &GameContext,
        char_screen: Point,
        mirror_h: bool,
    ) {
        let sprite = self.gift_sprite();
        let gift_w = sprite.width as i32;
        let gift_h = sprite.height as i32;
        let ground_y = char_screen.y - gift_h;
        let start_y = ground_y - 40;
        let gift_y = (start_y as f32 + (ground_y - start_y) as f32 * self.gift_y_progress) as i32;
        let gift_x = if mirror_h {
            char_screen.x + GIFT_OFFSET_X - gift_w / 2
        } else {
            char_screen.x - GIFT_OFFSET_X - gift_w / 2
        };
        renderer.draw_sprite(sprite, Point::new(gift_x, gift_y), SpriteOpts::default());
    }
}
