use crate::{
    context::GameContext,
    input::Buttons,
    render::Renderer,
    scenes::{menu_scene::MenuScene, ActiveScene},
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
    DebugWifi,
    DebugEspnow,
    Social,
    Zoomies,
    Breakout,
    Snake,
    Memory,
    Maze,
    Hanjie,
    TicTacToe,
    LightsOut,
    Pipes,
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

    /// Advance world state without consuming input or returning a scene swap.
    /// Called on the underlying scene while the big-menu overlay is open so
    /// timers, behaviors, and animations keep moving while the player navigates
    /// the menu. Default is a no-op (the scene pauses).
    fn tick_background(&mut self, _ctx: &mut GameContext, _dt: f32) {}
}

pub struct SceneManager {
    current: ActiveScene,
    current_id: SceneId,
    /// Big-menu overlay drawn over `current` without exiting it. Opened
    /// instantly (no transition) when the underlying scene requests
    /// `SceneId::Menu`; closed instantly when the menu is dismissed or
    /// when a chosen item matches the underlying scene.
    overlay: Option<MenuScene>,
    /// True after the overlay has dispatched a scene-swap action — input is
    /// frozen but the menu stays drawn so the player sees the fade-out
    /// covering the menu rather than briefly revealing the underlying scene.
    /// Cleared by `swap_to` when the transition midpoint actually fires.
    overlay_pending_close: bool,
}

impl SceneManager {
    pub fn new(ctx: &mut GameContext, start: SceneId) -> Self {
        let mut current = ActiveScene::from_id(start);
        current.as_scene_mut().enter(ctx);
        Self {
            current,
            current_id: start,
            overlay: None,
            overlay_pending_close: false,
        }
    }

    /// Tick the current scene and report any swap it requested.
    ///
    /// The swap is no longer applied inline — the caller (`Game`) defers it
    /// until the screen transition reaches its midpoint, so the player sees
    /// the fade-out → fade-in rather than an instant cut.
    ///
    /// `SceneId::Menu` requests open the big-menu overlay instead of being
    /// returned to the caller, so opening the menu doesn't fade or exit the
    /// underlying scene.
    pub fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        if let Some(overlay) = self.overlay.as_mut() {
            // Underlying scene keeps ticking (timers, animations, behaviors)
            // but doesn't receive input or get to request scene swaps.
            self.current.as_scene_mut().tick_background(ctx, dt);

            if self.overlay_pending_close {
                // A swap has already been dispatched; freeze the menu and let
                // the in-flight transition cover it before `swap_to` tears it
                // down at midpoint.
                return None;
            }

            let result = overlay.update(ctx, buttons, dt);
            if overlay.take_dismissed() {
                // Player pressed Menu1/Menu2/B to back out — close the overlay
                // and stay on the current scene (no transition).
                overlay.exit(ctx);
                self.overlay = None;
                return None;
            }
            if let Some(id) = result {
                if id == self.current_id {
                    overlay.exit(ctx);
                    self.overlay = None;
                    return None;
                }
                // Real swap requested — leave the menu drawn while the
                // transition plays; `swap_to` will close it at the midpoint.
                self.overlay_pending_close = true;
                return Some(id);
            }
            return None;
        }

        match self.current.as_scene_mut().update(ctx, buttons, dt) {
            Some(SceneId::Menu) => {
                let mut overlay = MenuScene::new();
                overlay.enter(ctx);
                self.overlay = Some(overlay);
                None
            }
            other => other,
        }
    }

    /// Apply a deferred scene swap. Called from `Game` at the transition
    /// midpoint while the screen is fully black.
    pub fn swap_to(&mut self, ctx: &mut GameContext, next: SceneId) {
        if let Some(mut overlay) = self.overlay.take() {
            overlay.exit(ctx);
        }
        self.overlay_pending_close = false;
        self.current.as_scene_mut().exit(ctx);
        self.current = ActiveScene::from_id(next);
        self.current_id = next;
        self.current.as_scene_mut().enter(ctx);
    }

    /// Minimal scene tick used by `SleepManager` while the screen is off.
    ///
    /// Mirrors Python `SceneManager.sleep_update`: ticks the current scene
    /// so behaviors and needs keep advancing, but ignores any returned
    /// scene-change request — switching scenes invisibly behind a black
    /// screen would surprise the player on wake. Behavior-requested scene
    /// changes mid-sleep are intentionally dropped.
    pub fn sleep_update(&mut self, ctx: &mut GameContext, buttons: &mut Buttons, dt: f32) {
        let _ = self.current.as_scene_mut().update(ctx, buttons, dt);
    }

    pub fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, dt_ms: u64) {
        if let Some(overlay) = self.overlay.as_ref() {
            overlay.draw(ctx, renderer, dt_ms);
        } else {
            self.current.as_scene().draw(ctx, renderer, dt_ms);
        }
    }
}
