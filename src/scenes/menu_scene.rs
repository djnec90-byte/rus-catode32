use embedded_graphics::prelude::*;

use crate::{
    context::GameContext,
    input::{Button, Buttons},
    render::Renderer,
    scene::{Scene, SceneId},
};

const ITEMS: [&str; 3] = ["Resume", "Reset stats", "Pose viewer"];

pub struct MenuScene {
    selected: usize,
}

impl MenuScene {
    pub fn new() -> Self {
        Self { selected: 0 }
    }
}

impl Scene for MenuScene {
    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::Up) && self.selected > 0 {
            self.selected -= 1;
        }
        if buttons.was_just_pressed(Button::Down) && self.selected < ITEMS.len() - 1 {
            self.selected += 1;
        }
        if buttons.was_just_pressed(Button::A) {
            match self.selected {
                0 => return Some(SceneId::Main),
                1 => {
                    *ctx = GameContext::new();
                    return Some(SceneId::Main);
                }
                2 => return Some(SceneId::PoseViewer),
                _ => {}
            }
        }
        if buttons.was_just_pressed(Button::B) || buttons.was_just_pressed(Button::Menu1) {
            return Some(SceneId::Main);
        }
        None
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        renderer.draw_text("Menu", Point::new(0, 0));

        for (i, label) in ITEMS.iter().enumerate() {
            let y = 16 + (i as i32) * 12;
            let marker = if i == self.selected { ">" } else { " " };
            renderer.draw_text(marker, Point::new(0, y));
            renderer.draw_text(label, Point::new(8, y));
        }

        renderer.draw_text("A:select  B:back", Point::new(0, 54));
    }
}
