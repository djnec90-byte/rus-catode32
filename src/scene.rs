use crate::{
    context::GameContext,
    input::Buttons,
    render::Renderer,
    scenes::ActiveScene,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SceneId {
    Inside,
    Outside,
    Bedroom,
    Kitchen,
    Treehouse,
    Menu,
    PoseViewer,
    Stats,
    Forecast,
    Store,
    Adoption,
    PetInfo,
    Credits,
    DebugBehaviors,
    DebugEnv,
    DebugStats,
    DebugTime,
    DebugLed,
    DebugPlants,
    DebugPower,
    DebugContext,
    Zoomies,
    Breakout,
    Snake,
    Memory,
    Platformer,
    VacationPark,
    VacationForest,
    VacationAquarium,
    VacationBeach,
    Stub(&'static str),
}

pub trait Scene {
    fn enter(&mut self, _ctx: &mut GameContext) {}
    fn exit(&mut self, _ctx: &mut GameContext) {}
    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId>;
    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, dt_ms: u64);
}

pub struct SceneManager {
    current: ActiveScene,
}

impl SceneManager {
    pub fn new(ctx: &mut GameContext, start: SceneId) -> Self {
        let mut current = ActiveScene::from_id(start);
        current.as_scene_mut().enter(ctx);
        Self { current }
    }

    /// Tick the current scene and report any swap it requested.
    ///
    /// The swap is no longer applied inline — the caller (`Game`) defers it
    /// until the screen transition reaches its midpoint, so the player sees
    /// the fade-out → fade-in rather than an instant cut.
    pub fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        self.current.as_scene_mut().update(ctx, buttons, dt)
    }

    /// Apply a deferred scene swap. Called from `Game` at the transition
    /// midpoint while the screen is fully black.
    pub fn swap_to(&mut self, ctx: &mut GameContext, next: SceneId) {
        self.current.as_scene_mut().exit(ctx);
        self.current = ActiveScene::from_id(next);
        self.current.as_scene_mut().enter(ctx);
    }

    /// Minimal scene tick used by `SleepManager` while the screen is off.
    ///
    /// Mirrors Python `SceneManager.sleep_update`: ticks the current scene
    /// so behaviors and needs keep advancing, but ignores any returned
    /// scene-change request — switching scenes invisibly behind a black
    /// screen would surprise the player on wake.
    ///
    /// TODO(sleep_pending_scene): Python defers scene changes triggered by
    /// behaviors during sleep via `ctx.pending_scene` + a wake-time
    /// `apply_pending_scene_after_sleep()` call. In Rust the location-scene
    /// update already `take()`s `pending_scene` inline, so a scene change
    /// requested mid-sleep is silently dropped. Fix once a sleep-aware
    /// scene-switch path exists.
    pub fn sleep_update(&mut self, ctx: &mut GameContext, buttons: &mut Buttons, dt: f32) {
        let _ = self.current.as_scene_mut().update(ctx, buttons, dt);
    }

    pub fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, dt_ms: u64) {
        self.current.as_scene().draw(ctx, renderer, dt_ms);
    }
}
