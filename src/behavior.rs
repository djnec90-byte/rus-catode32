use embedded_graphics::prelude::Point;

use crate::{
    assets::character::PoseId,
    behaviors::{auto_select, ActiveBehavior},
    context::{FoodKind, GameContext},
    entities::character::Character,
    render::Renderer,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviorId {
    Idle,
    Sleeping,
    Napping,
    Stretching,
    Kneading,
    Lounging,
    Investigating,
    Observing,
    Chattering,
    Zoomies,
    Vocalizing,
    SelfGrooming,
    BeingGroomed,
    Hunting,
    GiftBringing,
    Pacing,
    Sulking,
    Mischief,
    Hiding,
    Training,
    Playing,
    Affection,
    Attention,
    Eating,
    Startled,
    Meandering,
    GoTo,
    Hearing,
    Greeting,
}

#[allow(dead_code)]
impl BehaviorId {
    pub fn name(self) -> &'static str {
        match self {
            BehaviorId::Idle => "idle",
            BehaviorId::Sleeping => "sleeping",
            BehaviorId::Napping => "napping",
            BehaviorId::Stretching => "stretching",
            BehaviorId::Kneading => "kneading",
            BehaviorId::Lounging => "lounging",
            BehaviorId::Investigating => "investigating",
            BehaviorId::Observing => "observing",
            BehaviorId::Chattering => "chattering",
            BehaviorId::Zoomies => "zoomies",
            BehaviorId::Vocalizing => "vocalizing",
            BehaviorId::SelfGrooming => "self_grooming",
            BehaviorId::BeingGroomed => "being_groomed",
            BehaviorId::Hunting => "hunting",
            BehaviorId::GiftBringing => "gift_bringing",
            BehaviorId::Pacing => "pacing",
            BehaviorId::Sulking => "sulking",
            BehaviorId::Mischief => "mischief",
            BehaviorId::Hiding => "hiding",
            BehaviorId::Training => "training",
            BehaviorId::Playing => "playing",
            BehaviorId::Affection => "affection",
            BehaviorId::Attention => "attention",
            BehaviorId::Eating => "eating",
            BehaviorId::Startled => "startled",
            BehaviorId::Meandering => "meandering",
            BehaviorId::GoTo => "go_to",
            BehaviorId::Hearing => "hearing",
            BehaviorId::Greeting => "greeting",
        }
    }

    /// Interaction (player-initiated) behaviors that shouldn't auto-resume
    /// when re-entering a scene.
    pub fn is_interaction(self) -> bool {
        matches!(
            self,
            BehaviorId::Affection
                | BehaviorId::Attention
                | BehaviorId::BeingGroomed
                | BehaviorId::Eating
                | BehaviorId::Playing
                | BehaviorId::GiftBringing
                | BehaviorId::Chattering
                | BehaviorId::GoTo
                | BehaviorId::Hearing
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BehaviorState {
    Running,
    Completed,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlayVariant {
    Ball,
    String,
    Feather,
    Mouse,
    Hand,
    Laser,
    Bubbles,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AffectionVariant {
    Kiss,
    Pets,
    Scratching,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttentionVariant {
    Psst,
    PointBird,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrainingKind {
    Intelligence,
    Behavior,
    Fitness,
    Sociability,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GiftKind {
    Fish,
    Mouse,
}

/// What a behavior wants to chain to on natural completion. The manager turns
/// `None` into auto-select, and unknown variants into Idle.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum NextBehavior {
    Idle,
    Sleeping,
    Napping,
    Stretching,
    Kneading,
    Lounging,
    Investigating,
    Observing,
    Chattering,
    Zoomies,
    Vocalizing,
    SelfGrooming,
    Hunting,
    Pacing,
    Sulking,
    Mischief,
    Hiding,
    Meandering,
    Hearing,
    Startled,
    Greeting,
    Playing(PlayVariant),
    Affection(AffectionVariant),
    Attention(AttentionVariant),
    BeingGroomed,
    Eating(FoodKind),
    GiftBringing(GiftKind),
    Training(TrainingKind),
    GoTo(GoToParams),
}

#[derive(Clone, Copy, Debug)]
pub struct GoToParams {
    pub target_x: i32,
    pub speed: f32,
    pub pending_scene: Option<crate::scene::SceneId>,
    pub then: Option<GoToThen>,
}

/// A limited set of behaviors go_to can chain into on arrival. Kept flat so
/// `NextBehavior` doesn't need to be recursive.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum GoToThen {
    Sleeping,
    Napping,
    Lounging,
    Idle,
}

impl GoToThen {
    pub fn into_next(self) -> NextBehavior {
        match self {
            GoToThen::Sleeping => NextBehavior::Sleeping,
            GoToThen::Napping => NextBehavior::Napping,
            GoToThen::Lounging => NextBehavior::Lounging,
            GoToThen::Idle => NextBehavior::Idle,
        }
    }
}

#[allow(dead_code)]
pub trait Behavior {
    fn id(&self) -> BehaviorId;

    fn name(&self) -> &'static str {
        self.id().name()
    }

    fn progress(&self) -> f32;
    fn pose(&self) -> PoseId;

    fn enter(&mut self, _ctx: &mut GameContext, _character: &mut Character) {}
    fn update(
        &mut self,
        ctx: &mut GameContext,
        character: &mut Character,
        dt: f32,
    ) -> BehaviorState;
    fn exit(&mut self, _ctx: &mut GameContext, _completed: bool) {}

    fn apply_completion_bonus(&self, _ctx: &mut GameContext, _progress: f32) {}

    fn next(&self, _ctx: &GameContext) -> Option<NextBehavior> {
        None
    }

    /// Optional draw hook for particles, bubbles, item sprites, etc.
    /// `char_screen` is the on-screen character anchor; `mirror_h` matches the
    /// character so behavior visuals stay aligned with the pose.
    fn draw(
        &self,
        _renderer: &mut Renderer,
        _ctx: &GameContext,
        _char_screen: Point,
        _mirror_h: bool,
    ) {
    }

    /// Only `playing` uses this. Lets a behavior force a specific eye frame
    /// (e.g. eye-tracking a toy).
    fn eye_frame_override(&self) -> Option<usize> {
        None
    }

    /// Wind down quickly on wake-from-sleep. Only sleeping / napping override.
    fn mark_almost_done(&mut self) {}
}

pub struct BehaviorManager {
    current: ActiveBehavior,
    started: bool,
}

#[allow(dead_code)]
impl BehaviorManager {
    pub fn new() -> Self {
        Self {
            current: ActiveBehavior::from_next(NextBehavior::Idle),
            started: false,
        }
    }

    pub fn start(&mut self, ctx: &mut GameContext, character: &mut Character) {
        self.started = true;
        ctx.current_behavior_name = Some(self.current.as_dyn().name());
        self.current.as_dyn_mut().enter(ctx, character);
    }

    pub fn update(&mut self, ctx: &mut GameContext, character: &mut Character, dt: f32) {
        if !self.started {
            self.start(ctx, character);
        }
        let state = self.current.as_dyn_mut().update(ctx, character, dt);
        if state == BehaviorState::Completed {
            self.advance(ctx, character, true);
        }
    }

    /// Player-initiated trigger — interrupts the current behavior.
    pub fn trigger(
        &mut self,
        next: NextBehavior,
        ctx: &mut GameContext,
        character: &mut Character,
    ) {
        self.current.as_dyn_mut().exit(ctx, false);
        self.swap_to(next, ctx, character);
    }

    /// Skip the current behavior (debug / dev hook). Falls back to auto-select.
    pub fn skip(&mut self, ctx: &mut GameContext, character: &mut Character) {
        self.current.as_dyn_mut().exit(ctx, false);
        let next = auto_select(ctx);
        self.swap_to(next, ctx, character);
    }

    pub fn mark_almost_done(&mut self) {
        self.current.as_dyn_mut().mark_almost_done();
    }

    pub fn current_id(&self) -> BehaviorId {
        self.current.as_dyn().id()
    }

    pub fn current_name(&self) -> &'static str {
        self.current.as_dyn().name()
    }

    pub fn current_progress(&self) -> f32 {
        self.current.as_dyn().progress()
    }

    pub fn current_pose(&self) -> PoseId {
        self.current.as_dyn().pose()
    }

    pub fn current_eye_frame_override(&self) -> Option<usize> {
        self.current.as_dyn().eye_frame_override()
    }

    pub fn draw_overlay(
        &self,
        renderer: &mut Renderer,
        ctx: &GameContext,
        char_screen: Point,
        mirror_h: bool,
    ) {
        self.current
            .as_dyn()
            .draw(renderer, ctx, char_screen, mirror_h);
    }

    fn advance(&mut self, ctx: &mut GameContext, character: &mut Character, completed: bool) {
        let id = self.current.as_dyn().id();
        let progress = self.current.as_dyn().progress();
        let chained = if completed {
            self.current.as_dyn().next(ctx)
        } else {
            None
        };
        self.current.as_dyn_mut().exit(ctx, completed);
        if completed {
            ctx.record_behavior(id);
            apply_sickness_accumulation(ctx, id);
            self.current.as_dyn().apply_completion_bonus(ctx, progress);
        }

        let next = if let Some(n) = chained {
            n
        } else if completed {
            // Wake-from-sleep greeting takes precedence over auto-select.
            if ctx.pending_wake_greeting {
                ctx.pending_wake_greeting = false;
                if let Some(g) = wake_greeting(ctx) {
                    g
                } else {
                    auto_select(ctx)
                }
            } else {
                auto_select(ctx)
            }
        } else {
            auto_select(ctx)
        };

        self.swap_to(next, ctx, character);
    }

    fn swap_to(
        &mut self,
        next: NextBehavior,
        ctx: &mut GameContext,
        character: &mut Character,
    ) {
        self.current = ActiveBehavior::from_next(next);
        ctx.current_behavior_name = Some(self.current.as_dyn().name());
        self.current.as_dyn_mut().enter(ctx, character);
    }
}

fn apply_sickness_accumulation(ctx: &mut GameContext, completing: BehaviorId) {
    use crate::scene::SceneId;
    use crate::time_system::Weather;
    if matches!(completing, BehaviorId::Sleeping | BehaviorId::Napping) {
        return;
    }
    let outdoor = matches!(
        ctx.last_main_scene,
        SceneId::Outside | SceneId::Treehouse
    );
    let mut delta = 0.0;
    if outdoor {
        delta += match ctx.weather {
            Weather::Storm => 0.5,
            Weather::Rain | Weather::Snow => 0.25,
            _ => 0.0,
        };
    }
    if ctx.fullness < 10.0 {
        delta += 0.25;
    }
    if ctx.cleanliness < 15.0 {
        delta += 0.25;
    }
    if delta > 0.0 {
        ctx.sickness = (ctx.sickness + delta).min(10.0);
    }
}

fn wake_greeting(ctx: &GameContext) -> Option<NextBehavior> {
    if ctx.sickness >= 8.0 {
        return None;
    }
    const NEED: f32 = 50.0;
    let unmet = ctx.fullness < NEED
        || ctx.affection < NEED
        || ctx.comfort < NEED
        || ctx.fulfillment < NEED;
    let happy = ctx.energy > 40.0 && ctx.playfulness > 45.0;
    let mut rng = ctx.rng;
    let r = crate::rand::rand_f32(&mut rng);
    if unmet && r < 0.75 {
        Some(NextBehavior::Vocalizing)
    } else if happy && r < 0.55 {
        Some(NextBehavior::Vocalizing)
    } else {
        None
    }
}
