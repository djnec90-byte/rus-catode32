use core::fmt::Write;

use embedded_graphics::prelude::*;
use heapless::String;

use crate::{
    behavior::BehaviorManager,
    context::GameContext,
    input::{Button, Buttons},
    render::Renderer,
    scene::{Scene, SceneId},
};

const DISPLAYED_STATS: usize = 2;

pub struct MainScene {
    focused_stat: usize,
    behaviors: BehaviorManager,
}

impl MainScene {
    pub fn new() -> Self {
        Self {
            focused_stat: 0,
            behaviors: BehaviorManager::new(),
        }
    }
}

impl Scene for MainScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.behaviors.start(ctx);
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::Up) && self.focused_stat > 0 {
            self.focused_stat -= 1;
        }
        if buttons.was_just_pressed(Button::Down)
            && self.focused_stat < DISPLAYED_STATS - 1
        {
            self.focused_stat += 1;
        }
        if buttons.was_just_pressed(Button::A) {
            adjust_stat(ctx, self.focused_stat, 10.0);
        }
        if buttons.was_just_pressed(Button::B) {
            adjust_stat(ctx, self.focused_stat, -10.0);
        }
        if buttons.was_just_pressed(Button::Menu2) {
            self.behaviors.skip(ctx);
        }
        if buttons.was_just_pressed(Button::Menu1) {
            return Some(SceneId::Menu);
        }

        self.behaviors.update(ctx, dt);
        None
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, dt_ms: u64) {
        let mut title: String<32> = String::new();
        write!(title, "catode32 v0.10  dt:{}", dt_ms).ok();
        renderer.draw_text(title.as_str(), Point::new(0, 0));

        let labels = ["Full", "Enrg"];
        let values = [ctx.fullness, ctx.energy];

        for i in 0..DISPLAYED_STATS {
            let y = 14 + (i as i32) * 12;
            let marker = if i == self.focused_stat { ">" } else { " " };
            renderer.draw_text(marker, Point::new(0, y));
            renderer.draw_text(labels[i], Point::new(6, y));

            let mut val: String<8> = String::new();
            write!(val, "{:>3}", values[i] as i32).ok();
            renderer.draw_text(val.as_str(), Point::new(32, y));

            draw_bar(renderer, 54, y + 2, 70, 6, values[i]);
        }

        let mut beh: String<32> = String::new();
        write!(beh, "Behavior: {}", self.behaviors.current_name()).ok();
        renderer.draw_text(beh.as_str(), Point::new(0, 42));

        let progress = (self.behaviors.current_progress() * 100.0).clamp(0.0, 100.0);
        draw_bar(renderer, 2, 54, 124, 6, progress);
    }
}

fn draw_bar(renderer: &mut Renderer, x: i32, y: i32, w: u32, h: u32, value_0_100: f32) {
    renderer.draw_rect(Point::new(x, y), Size::new(w, h), false);
    let fill = (value_0_100.clamp(0.0, 100.0) / 100.0 * w as f32) as u32;
    if fill > 0 {
        renderer.draw_rect(Point::new(x, y), Size::new(fill, h), true);
    }
}

fn adjust_stat(ctx: &mut GameContext, idx: usize, delta: f32) {
    let s = displayed_stat_mut(ctx, idx);
    *s = (*s + delta).clamp(0.0, 100.0);
}

fn displayed_stat_mut(ctx: &mut GameContext, idx: usize) -> &mut f32 {
    match idx {
        0 => &mut ctx.fullness,
        1 => &mut ctx.energy,
        _ => unreachable!(),
    }
}
