//! Shared vacation timer + stat-accrual / overstay-penalty logic, plus the
//! [`VacationScene`] / [`VacationWorld`] split that lets each destination
//! describe only the bits unique to it (geometry, art, entity state) and
//! inherit the boilerplate `Scene` impl from this module.

use embedded_graphics::prelude::Point;

use crate::{
    context::{GameContext, StatId},
    gardening_ui::PlantSurface,
    input::Buttons,
    location_scene::LocationScene,
    render::Renderer,
    scene::{Scene, SceneId},
};

#[derive(Clone, Copy)]
pub struct VacationConfig {
    /// Seconds until the enjoyment cap is reached and the pet starts asking
    /// to go home.
    pub enjoy_duration: f32,
    /// Seconds of grace after the cap before overstay penalties begin.
    pub grace_duration: f32,
    /// Stat totals awarded on exit, scaled to time spent up to the cap.
    pub accrual: &'static [(StatId, f32)],
    /// Per-second penalties applied during overstay (negative deltas).
    pub penalties: &'static [(StatId, f32)],
}

/// Default enjoyment cap shared by every Python `VacationScene` subclass; the
/// `VacationConfig::standard` helper bakes this in so destinations only have
/// to declare their accrual stats.
pub const STANDARD_ENJOY_DURATION: f32 = 750.0;
/// Default post-cap grace before overstay penalties begin (matches Python).
pub const STANDARD_GRACE_DURATION: f32 = 120.0;
/// Default overstay penalty stat-pair (matches Python's `STAT_PENALTIES`
/// default on the `VacationScene` base class; no subclass overrides it).
pub const STANDARD_PENALTIES: &[(StatId, f32)] = &[
    (StatId::Comfort, -0.005),
    (StatId::Serenity, -0.003),
];

impl VacationConfig {
    /// Most destinations share `STANDARD_ENJOY_DURATION` / `STANDARD_GRACE_DURATION`
    /// / `STANDARD_PENALTIES` and only differ in the accrual stat-pair.
    pub const fn standard(accrual: &'static [(StatId, f32)]) -> Self {
        Self {
            enjoy_duration: STANDARD_ENJOY_DURATION,
            grace_duration: STANDARD_GRACE_DURATION,
            accrual,
            penalties: STANDARD_PENALTIES,
        }
    }
}

const PENALTY_INTERVAL: f32 = 90.0;

pub struct VacationState {
    pub config: VacationConfig,
    timer: f32,
    home_wanted: bool,
    penalty_accum: f32,
}

impl VacationState {
    pub fn new(config: VacationConfig) -> Self {
        Self {
            config,
            timer: 0.0,
            home_wanted: false,
            penalty_accum: 0.0,
        }
    }

    /// Stamps the cross-scene flags so the behavior layer (auto-pick
    /// scene-exit, vocalizing) sees them.
    pub fn on_enter(&mut self, ctx: &mut GameContext) {
        self.timer = 0.0;
        self.home_wanted = false;
        self.penalty_accum = 0.0;
        ctx.on_vacation = true;
        ctx.wants_to_go_home = false;
    }

    /// Lands the proportional reward (only if any time was spent) and clears
    /// the vacation flags so the next scene's behavior cycle is unaffected.
    pub fn on_exit(&mut self, ctx: &mut GameContext) {
        self.apply_rewards(ctx);
        ctx.on_vacation = false;
        ctx.wants_to_go_home = false;
    }

    pub fn tick(&mut self, ctx: &mut GameContext, dt: f32) {
        let prev = self.timer;
        self.timer += dt;

        // Crossed the enjoyment cap this frame, start the "go home" nag.
        if prev < self.config.enjoy_duration && self.timer >= self.config.enjoy_duration {
            self.home_wanted = true;
            ctx.wants_to_go_home = true;
        }

        // Overstay penalties: accumulate dt and apply in batches so the
        // penalty fires noticeably rather than being lost to per-frame
        // damping inside apply_stat_changes.
        if !self.config.penalties.is_empty()
            && self.timer >= self.config.enjoy_duration + self.config.grace_duration
        {
            self.penalty_accum += dt;
            if self.penalty_accum >= PENALTY_INTERVAL {
                let scale = self.penalty_accum;
                self.penalty_accum = 0.0;
                let mut batched: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
                for &(stat, per_sec) in self.config.penalties {
                    let _ = batched.push((stat, per_sec * scale));
                }
                ctx.apply_stat_changes(&batched);
            }
        }
    }

    fn apply_rewards(&self, ctx: &mut GameContext) {
        if self.config.accrual.is_empty() {
            return;
        }
        let proportion = (self.timer / self.config.enjoy_duration).clamp(0.0, 1.0);
        if proportion <= 0.0 {
            return;
        }
        let mut batched: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
        for &(stat, total) in self.config.accrual {
            let _ = batched.push((stat, total * proportion));
        }
        ctx.apply_stat_changes(&batched);
    }
}

// ---------------------------------------------------------------------------
// Vacation Scene generic wrapper
// ---------------------------------------------------------------------------

/// Per-destination contract. The wrapper [`VacationScene`] handles the
/// `Scene`-trait boilerplate (enter / exit flag-stamping, dt scaling, menu
/// gating, character + overlay draws); each implementor only declares the
/// geometry, art, and per-frame world updates that are actually unique.
///
/// All four current destinations use the same enjoy/grace/penalty defaults
/// from [`VacationConfig::standard`].
pub trait VacationWorld: Default {
    /// `SceneId` this destination identifies as.
    const SCENE_ID: SceneId;
    /// World width in pixels.
    const WORLD_WIDTH: i32;
    /// Initial character world x.
    const CHAR_WORLD_X: i32;
    /// Initial character world y.
    const GROUND_Y: i32;
    /// Left bound of the walkable strip (sets `ctx.scene_x_min`).
    const X_MIN: i32;
    /// Right bound of the walkable strip (sets `ctx.scene_x_max`).
    const X_MAX: i32;
    /// Vacation timer/accrual/penalty config.
    const CONFIG: VacationConfig;
    /// True for outdoor destinations that draw the sky; false for indoor
    /// (e.g. aquarium).
    const HAS_SKY: bool;
    /// Plant surfaces this scene exposes. Most vacation scenes have none.
    const PLANT_SURFACES: &'static [PlantSurface] = &[];

    /// One-shot setup: spawn entities, place environment objects, reseed rng.
    /// Called after the base scene has finished its own enter and the
    /// vacation flags have been stamped.
    fn enter(&mut self, _ctx: &mut GameContext, _base: &mut LocationScene) {}

    /// Optional teardown for per-world transient state. The vacation flags
    /// and reward landing are handled by the wrapper.
    fn exit(&mut self, _ctx: &mut GameContext) {}

    /// Per-frame world advance. `dt` is the frame delta already scaled by
    /// `ctx.time_speed` (all scaling happens once in `Game::update`). The
    /// wrapper ticks the shared `VacationState` afterwards.
    fn tick(&mut self, _ctx: &mut GameContext, _dt: f32) {}

    /// Draw the destination's art (environment layers + custom passes) between
    /// the optional sky and the character. The wrapper handles menu gating,
    /// sky (if `HAS_SKY`), character, and overlay.
    fn draw_world(&self, ctx: &GameContext, renderer: &mut Renderer, base: &LocationScene);
}

pub struct VacationScene<W: VacationWorld> {
    base: LocationScene,
    state: VacationState,
    world: W,
}

impl<W: VacationWorld> VacationScene<W> {
    pub fn new() -> Self {
        Self {
            base: LocationScene::new(
                W::WORLD_WIDTH,
                Point::new(W::CHAR_WORLD_X, W::GROUND_Y),
            ),
            state: VacationState::new(W::CONFIG),
            world: W::default(),
        }
    }
}

impl<W: VacationWorld> Scene for VacationScene<W> {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.base.enter(ctx, W::SCENE_ID, W::PLANT_SURFACES);
        ctx.scene_x_min = W::X_MIN;
        ctx.scene_x_max = W::X_MAX;
        self.world.enter(ctx, &mut self.base);
        self.state.on_enter(ctx);
    }

    fn exit(&mut self, ctx: &mut GameContext) {
        self.state.on_exit(ctx);
        self.world.exit(ctx);
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        if let Some(id) = self.base.update(ctx, buttons, dt) {
            return Some(id);
        }
        self.world.tick(ctx, dt);
        self.state.tick(ctx, dt);
        None
    }

    fn tick_background(&mut self, ctx: &mut GameContext, dt: f32) {
        self.base.tick_background(ctx, dt);
        self.world.tick(ctx, dt);
        self.state.tick(ctx, dt);
    }

    fn mark_behavior_almost_done(&mut self, ctx: &mut GameContext) {
        self.base.mark_behavior_almost_done(ctx);
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        if self.base.menu_active() {
            self.base.draw_menu(renderer);
            return;
        }
        if W::HAS_SKY {
            self.base.draw_sky(renderer, ctx);
        }
        self.world.draw_world(ctx, renderer, &self.base);
        self.base.draw_character(renderer, ctx);
        self.base.draw_overlay(ctx, renderer);
    }
}
