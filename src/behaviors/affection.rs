use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{AffectionVariant, Behavior, BehaviorId, BehaviorState},
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
    ui::bubble::{self, BubbleIcon},
};

pub struct AffectionBehavior {
    variant: AffectionVariant,
    phase_timer: f32,
    elapsed: f32,
    total: f32,
    pose_id: PoseId,
    rejected: bool,
}

impl AffectionBehavior {
    #[allow(dead_code)]
    pub fn new(variant: AffectionVariant) -> Self {
        Self {
            variant,
            phase_timer: 0.0,
            elapsed: 0.0,
            total: 5.0,
            pose_id: PoseId::SittingForwardHappy,
            rejected: false,
        }
    }
}

impl Behavior for AffectionBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Affection
    }
    fn progress(&self) -> f32 {
        (self.elapsed / self.total).clamp(0.0, 1.0)
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.rejected = ctx.affection > 80.0 && ctx.serenity < 30.0;
        self.phase_timer = 0.0;
        self.elapsed = 0.0;
        self.total = rand::rand_range_f32(
            &mut ctx.rng,
            match self.variant {
                AffectionVariant::Kiss => 3.0,
                AffectionVariant::Pets => 5.0,
                AffectionVariant::Scratching => 6.0,
            },
            match self.variant {
                AffectionVariant::Kiss => 4.0,
                AffectionVariant::Pets => 7.0,
                AffectionVariant::Scratching => 8.0,
            },
        );
        self.pose_id = if self.rejected {
            PoseId::LayingSideAnnoyed
        } else if ctx.sickness >= 2.0 {
            PoseId::LayingSideSick
        } else {
            match self.variant {
                AffectionVariant::Kiss => PoseId::SittingForwardHappy,
                AffectionVariant::Pets => PoseId::LayingSideContent,
                AffectionVariant::Scratching => PoseId::LayingSideBliss,
            }
        };
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.elapsed += dt;
        self.phase_timer += dt;
        if self.elapsed >= self.total {
            BehaviorState::Completed
        } else {
            BehaviorState::Running
        }
    }

    fn exit(&mut self, ctx: &mut GameContext, completed: bool) {
        if completed {
            ctx.milestone_petted = true;
        }
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mult = if self.rejected { 0.5 } else { 1.0 };
        let (aff, ser, soc) = match self.variant {
            AffectionVariant::Kiss => (6.0, 1.0, 0.4),
            AffectionVariant::Pets => (9.0, 1.5, 0.5),
            AffectionVariant::Scratching => (10.0, 2.0, 0.6),
        };
        let bonus = [
            (StatId::Affection, aff * mult * progress),
            (StatId::Serenity, ser * mult * progress),
            (StatId::Sociability, soc * mult * progress),
            (StatId::Comfort, 2.0 * mult * progress),
        ];
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(&self, renderer: &mut Renderer, _ctx: &GameContext, char_screen: Point, mirror_h: bool) {
        if self.rejected {
            return;
        }
        bubble::draw_above_char(
            renderer,
            BubbleIcon::Heart,
            char_screen.x,
            char_screen.y,
            self.progress(),
            mirror_h,
        );
    }
}
