use embedded_graphics::prelude::Point;
use esp_hal::time::Instant;

use crate::{
    assets::character::{PoseId, ALL_POSES},
    character::{draw_pose, pose_layout, PoseAnim},
    context::GameContext,
    input::{Button, Buttons},
    render::Renderer,
    scene::{Scene, SceneId},
};

const CHAR_X: i32 = 64;
const CHAR_Y: i32 = 60;
const FLOOR_Y: i32 = 60;
const GRID_SPACING: i32 = 8;
const GRID_SPEED: f32 = 16.0;

pub struct PoseScene {
    pose_index: usize,
    show_anchors: bool,
    show_grid: bool,
    grid_offset: f32,
    anim: PoseAnim,
}

impl PoseScene {
    pub fn new() -> Self {
        Self {
            pose_index: 0,
            show_anchors: false,
            show_grid: false,
            grid_offset: 0.0,
            anim: PoseAnim::new(1),
        }
    }

    fn current_pose(&self) -> PoseId {
        ALL_POSES[self.pose_index]
    }
}

impl Scene for PoseScene {
    fn enter(&mut self, _ctx: &mut GameContext) {
        let seed = (Instant::now().duration_since_epoch().as_micros() as u32).max(1);
        self.anim = PoseAnim::new(seed);
        self.anim.reseed_for(self.current_pose().data());
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::Left) {
            self.pose_index = if self.pose_index == 0 {
                ALL_POSES.len() - 1
            } else {
                self.pose_index - 1
            };
            self.anim.reseed_for(self.current_pose().data());
        }
        if buttons.was_just_pressed(Button::Right) {
            self.pose_index = (self.pose_index + 1) % ALL_POSES.len();
            self.anim.reseed_for(self.current_pose().data());
        }
        if buttons.was_just_pressed(Button::Up) {
            self.show_anchors = !self.show_anchors;
        }
        if buttons.was_just_pressed(Button::Down) {
            self.show_grid = !self.show_grid;
        }
        if buttons.was_just_pressed(Button::B) || buttons.was_just_pressed(Button::Menu1) {
            return Some(ctx.last_main_scene);
        }

        self.anim.update(self.current_pose().data(), dt);
        if self.show_grid {
            self.grid_offset = (self.grid_offset + dt * GRID_SPEED) % GRID_SPACING as f32;
        }
        None
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        if self.show_grid {
            let offset = self.grid_offset as i32;
            let mut x = -GRID_SPACING + offset;
            while x <= 128 + GRID_SPACING {
                renderer.draw_line(Point::new(x, 0), Point::new(x, 64));
                x += GRID_SPACING;
            }
            let mut y = -GRID_SPACING + offset;
            while y <= 64 + GRID_SPACING {
                renderer.draw_line(Point::new(0, y), Point::new(128, y));
                y += GRID_SPACING;
            }
        }

        renderer.draw_line(Point::new(0, FLOOR_Y), Point::new(128, FLOOR_Y));

        let pose = self.current_pose();
        let name = pose.name();
        for (i, part) in name.split('.').enumerate() {
            renderer.draw_text(part, Point::new(0, (i as i32) * 8));
        }

        let pose_data = pose.data();
        draw_pose(
            renderer,
            pose_data,
            &self.anim,
            Point::new(CHAR_X, CHAR_Y),
            false,
        );

        if self.show_anchors {
            let layout = pose_layout(pose_data, Point::new(CHAR_X, CHAR_Y), false);
            draw_anchor_rect(renderer, CHAR_X, CHAR_Y);
            draw_anchor_rect(renderer, layout.head_attach.0 as i32, layout.head_attach.1 as i32);
            draw_anchor_rect(renderer, layout.tail_attach.0 as i32, layout.tail_attach.1 as i32);
            draw_x_marker(renderer, layout.head_attach.0 as i32, layout.head_attach.1 as i32);
            draw_x_marker(renderer, layout.tail_attach.0 as i32, layout.tail_attach.1 as i32);
            if let Some(attach) = layout.eye_attach {
                draw_anchor_rect(renderer, attach.0 as i32, attach.1 as i32);
                draw_x_marker(renderer, attach.0 as i32, attach.1 as i32);
            }
        }
    }
}

fn draw_anchor_rect(renderer: &mut Renderer, cx: i32, cy: i32) {
    for dx in -2..=2 {
        for dy in -2..=2 {
            renderer.draw_pixel(Point::new(cx + dx, cy + dy), true);
        }
    }
    for dx in -1..=1 {
        for dy in -1..=1 {
            renderer.draw_pixel(Point::new(cx + dx, cy + dy), false);
        }
    }
}

fn draw_x_marker(renderer: &mut Renderer, cx: i32, cy: i32) {
    renderer.draw_pixel(Point::new(cx - 1, cy - 1), true);
    renderer.draw_pixel(Point::new(cx, cy), true);
    renderer.draw_pixel(Point::new(cx + 1, cy + 1), true);
    renderer.draw_pixel(Point::new(cx + 1, cy - 1), true);
    renderer.draw_pixel(Point::new(cx - 1, cy + 1), true);
}
