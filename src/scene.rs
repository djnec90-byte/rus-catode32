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

    pub fn update(&mut self, ctx: &mut GameContext, buttons: &mut Buttons, dt: f32) {
        if let Some(next) = self.current.as_scene_mut().update(ctx, buttons, dt) {
            self.current.as_scene_mut().exit(ctx);
            self.current = ActiveScene::from_id(next);
            self.current.as_scene_mut().enter(ctx);
        }
    }

    pub fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, dt_ms: u64) {
        self.current.as_scene().draw(ctx, renderer, dt_ms);
    }
}
