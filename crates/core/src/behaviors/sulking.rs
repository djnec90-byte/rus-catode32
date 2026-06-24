use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behavior::{Behavior, BehaviorId, BehaviorState, NextBehavior},
    behaviors::common,
    context::{GameContext, StatId},
    entities::character::Character,
    rand,
    render::Renderer,
    ui::bubble::{self, BubbleIcon},
};

const BUBBLE_DURATION: f32 = 3.5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Settling,
    Sulking,
    Emerging,
}

pub struct SulkingBehavior {
    phase: Phase,
    phase_timer: f32,
    settle_duration: f32,
    sulk_duration: f32,
    emerge_duration: f32,
    pose_id: PoseId,
    sulk_pose: PoseId,
    sulk_cause: BubbleIcon,
    bubble_trigger_time: f32,
    bubble_timer: Option<f32>,
}

impl SulkingBehavior {
    pub fn new() -> Self {
        Self {
            phase: Phase::Settling,
            phase_timer: 0.0,
            settle_duration: 3.0,
            sulk_duration: 30.0,
            emerge_duration: 3.0,
            pose_id: PoseId::SittingSideAloof,
            sulk_pose: PoseId::LayingSideBored,
            sulk_cause: BubbleIcon::Lonely,
            bubble_trigger_time: 0.0,
            bubble_timer: None,
        }
    }

    pub fn can_trigger(ctx: &GameContext) -> bool {
        let stats = [ctx.fullness, ctx.affection, ctx.fulfillment, ctx.comfort];
        let low = stats.iter().filter(|v| **v < 50.0).count();
        ctx.fulfillment < 50.0
            || ctx.affection < 50.0
            || low >= 2
            || stats.iter().any(|v| *v < 25.0)
    }

    pub fn priority(ctx: &GameContext, rng: &mut u32) -> u32 {
        let combined = ctx.fulfillment + ctx.affection + ctx.fullness + ctx.comfort;
        let low = [ctx.fullness, ctx.affection, ctx.fulfillment, ctx.comfort]
            .iter()
            .filter(|v| **v < 50.0)
            .count() as f32;
        let ceiling = (combined * 0.225 - low * 5.0).max(10.0);
        let mut base = rand::rand_range_f32(rng, 10.0, ceiling.max(10.0));
        if !ctx.in_familiar_location {
            base *= 0.85;
        }
        base.max(0.0) as u32
    }

    /// Pick the bubble icon for the dominant unmet need (lowest stat wins).
    fn pick_sulk_cause(ctx: &GameContext) -> BubbleIcon {
        let needs = [
            (ctx.fullness, BubbleIcon::Hunger),
            (ctx.comfort, BubbleIcon::Discomfort),
            (ctx.fulfillment, BubbleIcon::Bored),
            (ctx.affection, BubbleIcon::Lonely),
        ];
        needs
            .iter()
            .copied()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal))
            .map(|x| x.1)
            .unwrap_or(BubbleIcon::Lonely)
    }

    /// Weighted pick from the 3-pose pool based on accumulated unmet-needs distress.
    fn pick_sulk_pose(ctx: &mut GameContext) -> PoseId {
        let distress = ((50.0 - ctx.fullness).max(0.0)
            + (50.0 - ctx.affection).max(0.0)
            + (50.0 - ctx.comfort).max(0.0)
            + (50.0 - ctx.fulfillment).max(0.0))
            / 200.0;
        let w_bored = (1.0 - distress * 2.0).max(0.0);
        let w_sulking = 1.0_f32;
        let w_sulking2 = (distress * 2.0 - 0.5).max(0.0);
        let total = w_bored + w_sulking + w_sulking2;
        let mut r = rand::rand_range_f32(&mut ctx.rng, 0.0, total);
        if r < w_bored {
            return PoseId::LayingSideBored;
        }
        r -= w_bored;
        if r < w_sulking {
            return PoseId::LayingSideSulking;
        }
        PoseId::LayingSideSulking2
    }
}

impl Behavior for SulkingBehavior {
    fn id(&self) -> BehaviorId {
        BehaviorId::Sulking
    }
    fn progress(&self) -> f32 {
        match self.phase {
            Phase::Settling => 0.0,
            Phase::Sulking => (self.phase_timer / self.sulk_duration).clamp(0.0, 1.0),
            Phase::Emerging => 1.0,
        }
    }
    fn pose(&self) -> PoseId {
        self.pose_id
    }

    fn enter(&mut self, ctx: &mut GameContext, _: &mut Character) {
        self.phase = Phase::Settling;
        self.phase_timer = 0.0;
        self.settle_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 5.0);
        self.sulk_duration = rand::rand_range_f32(&mut ctx.rng, 20.0, 45.0);
        self.emerge_duration = rand::rand_range_f32(&mut ctx.rng, 1.0, 5.0);
        self.sulk_pose = Self::pick_sulk_pose(ctx);
        self.sulk_cause = Self::pick_sulk_cause(ctx);
        self.bubble_trigger_time = rand::rand_range_f32(
            &mut ctx.rng,
            self.sulk_duration * 0.2,
            self.sulk_duration * 0.7,
        );
        self.bubble_timer = None;
        self.pose_id = PoseId::SittingSideAloof;
    }

    fn update(&mut self, _ctx: &mut GameContext, _: &mut Character, dt: f32) -> BehaviorState {
        self.phase_timer += dt;
        match self.phase {
            Phase::Settling if self.phase_timer >= self.settle_duration => {
                self.phase = Phase::Sulking;
                self.phase_timer = 0.0;
                self.pose_id = self.sulk_pose;
            }
            Phase::Sulking => {
                if self.bubble_timer.is_none()
                    && self.phase_timer >= self.bubble_trigger_time
                {
                    self.bubble_timer = Some(0.0);
                }
                if let Some(t) = self.bubble_timer.as_mut() {
                    if *t < BUBBLE_DURATION {
                        *t += dt;
                    }
                }
                if self.phase_timer >= self.sulk_duration {
                    self.phase = Phase::Emerging;
                    self.phase_timer = 0.0;
                    self.pose_id = PoseId::SittingSideNeutral;
                }
            }
            Phase::Emerging if self.phase_timer >= self.emerge_duration => {
                return BehaviorState::Completed;
            }
            _ => {}
        }
        BehaviorState::Running
    }

    fn next(&self, _ctx: &GameContext) -> Option<NextBehavior> {
        Some(NextBehavior::Pacing)
    }

    fn apply_completion_bonus(&self, ctx: &mut GameContext, progress: f32) {
        let mut bonus: heapless::Vec<(StatId, f32), 10> = heapless::Vec::new();
        common::bonus_add(&mut bonus, StatId::Comfort, 0.2);
        common::bonus_add(&mut bonus, StatId::Affection, -0.025);
        common::bonus_add(&mut bonus, StatId::Maturity, -0.025);
        common::bonus_add(&mut bonus, StatId::Sociability, -0.1);
        common::bonus_add(&mut bonus, StatId::Loyalty, -0.02);
        common::bonus_add(&mut bonus, StatId::Courage, -0.005);

        let hf = common::hungry_factor(ctx);
        if hf > 0.0 {
            common::bonus_add(&mut bonus, StatId::Loyalty, -0.05 * hf);
            common::bonus_add(&mut bonus, StatId::Affection, -0.05 * hf);
            common::bonus_add(&mut bonus, StatId::Serenity, -0.75 * hf);
            common::bonus_add(&mut bonus, StatId::Fulfillment, -0.05 * hf);
        }

        for e in bonus.iter_mut() {
            e.1 *= progress;
        }
        ctx.apply_stat_changes(&bonus);
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _ctx: &GameContext,
        char_screen: Point,
        mirror_h: bool,
    ) {
        if self.phase != Phase::Sulking {
            return;
        }
        let Some(t) = self.bubble_timer else { return };
        if t >= BUBBLE_DURATION {
            return;
        }
        let progress = (t / BUBBLE_DURATION).clamp(0.0, 1.0);
        bubble::draw_above_char(
            renderer,
            self.sulk_cause,
            char_screen.x,
            char_screen.y,
            progress,
            mirror_h,
        );
    }
}
