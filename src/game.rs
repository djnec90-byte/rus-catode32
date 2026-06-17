use core::fmt::Write;

use embedded_graphics::prelude::*;
use esp_hal::time::{Duration, Instant};
use heapless::String;

use crate::{
    context::GameContext,
    input::{Button, Buttons},
    render::Renderer,
};

const FPS: u64 = 12;
const FRAME_TIME_MS: u64 = 1000 / FPS;
const DISPLAYED_STATS: usize = 5;

pub struct Game {
    renderer: Renderer,
    buttons: Buttons,
    context: GameContext,
    focused_stat: usize,
    last_dt_ms: u64,
}

impl Game {
    pub fn new(renderer: Renderer, buttons: Buttons) -> Self {
        Self {
            renderer,
            buttons,
            context: GameContext::new(),
            focused_stat: 0,
            last_dt_ms: 0,
        }
    }

    pub fn run(&mut self) -> ! {
        let mut last_frame = Instant::now();
        loop {
            let frame_start = Instant::now();
            let elapsed_ms = (frame_start.duration_since_epoch()
                - last_frame.duration_since_epoch())
            .as_millis();
            last_frame = frame_start;
            self.last_dt_ms = elapsed_ms;

            let dt = elapsed_ms as f32 / 1000.0;
            self.update(dt);
            self.draw();

            let frame_used = frame_start.elapsed().as_millis();
            if frame_used < FRAME_TIME_MS {
                let wait_start = Instant::now();
                let remaining = Duration::from_millis(FRAME_TIME_MS - frame_used);
                while wait_start.elapsed() < remaining {}
            }
        }
    }

    fn update(&mut self, dt: f32) {
        if self.buttons.was_just_pressed(Button::Up) && self.focused_stat > 0 {
            self.focused_stat -= 1;
        }
        if self.buttons.was_just_pressed(Button::Down)
            && self.focused_stat < DISPLAYED_STATS - 1
        {
            self.focused_stat += 1;
        }
        if self.buttons.was_just_pressed(Button::A) {
            self.adjust_focused(10.0);
        }
        if self.buttons.was_just_pressed(Button::B) {
            self.adjust_focused(-10.0);
        }
        self.context.tick(dt);
    }

    fn adjust_focused(&mut self, delta: f32) {
        let s = displayed_stat_mut(&mut self.context, self.focused_stat);
        *s = (*s + delta).clamp(0.0, 100.0);
    }

    fn draw(&mut self) {
        self.renderer.clear();

        let mut title: String<32> = String::new();
        write!(title, "catode32 v0.10  dt:{}", self.last_dt_ms).ok();
        self.renderer.draw_text(title.as_str(), Point::new(0, 0));

        let labels = ["Full", "Enrg", "Cmft", "Play", "Focs"];
        let values = [
            self.context.fullness,
            self.context.energy,
            self.context.comfort,
            self.context.playfulness,
            self.context.focus,
        ];

        for i in 0..DISPLAYED_STATS {
            let y = 14 + (i as i32) * 10;
            let marker = if i == self.focused_stat { ">" } else { " " };
            self.renderer.draw_text(marker, Point::new(0, y));
            self.renderer.draw_text(labels[i], Point::new(6, y));

            let mut val: String<8> = String::new();
            write!(val, "{:>3}", values[i] as i32).ok();
            self.renderer.draw_text(val.as_str(), Point::new(32, y));

            let bar_x = 54;
            let bar_w: u32 = 70;
            let bar_h: u32 = 6;
            let bar_y = y + 2;
            self.renderer
                .draw_rect(Point::new(bar_x, bar_y), Size::new(bar_w, bar_h), false);
            let fill = (values[i].clamp(0.0, 100.0) / 100.0 * bar_w as f32) as u32;
            if fill > 0 {
                self.renderer
                    .draw_rect(Point::new(bar_x, bar_y), Size::new(fill, bar_h), true);
            }
        }

        self.renderer.flush();
    }
}

fn displayed_stat_mut(ctx: &mut GameContext, idx: usize) -> &mut f32 {
    match idx {
        0 => &mut ctx.fullness,
        1 => &mut ctx.energy,
        2 => &mut ctx.comfort,
        3 => &mut ctx.playfulness,
        4 => &mut ctx.focus,
        _ => unreachable!(),
    }
}
