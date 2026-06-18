use embedded_graphics::prelude::Point;

use crate::{
    context::GameContext,
    input::{Button, Buttons},
    render::Renderer,
    scene::{Scene, SceneId},
};

pub struct StubScene {
    name: &'static str,
}

impl StubScene {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl Scene for StubScene {
    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::B) || buttons.was_just_pressed(Button::Menu1) {
            return Some(ctx.last_main_scene);
        }
        None
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        renderer.draw_text("Not yet ported:", Point::new(0, 16));
        renderer.draw_text(self.name, Point::new(0, 28));
        renderer.draw_text("[B] Back", Point::new(0, 50));
    }
}
