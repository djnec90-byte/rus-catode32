use embedded_graphics::prelude::{Point, Size};

use crate::{
    assets::furniture::BOOKSHELF,
    context::GameContext,
    environment::Layer,
    input::{Button, Buttons},
    location_scene::LocationScene,
    render::Renderer,
    scene::{Scene, SceneId},
};

const WORLD_WIDTH: i32 = 192;
const CHAR_WORLD_X: i32 = 64;
const CHAR_WORLD_Y: i32 = 64;

const DISPLAY_WIDTH: i32 = 128;
const DISPLAY_HEIGHT: i32 = 64;

// Window cutout — mirrors Python's InsideScene constants exactly.
const WINDOW_WORLD_X: i32 = 100;
const WINDOW_Y: i32 = -10;
const WINDOW_W: i32 = 56;
const WINDOW_H: i32 = 36;

pub struct InsideScene {
    base: LocationScene,
}

impl InsideScene {
    pub fn new() -> Self {
        Self {
            base: LocationScene::new(WORLD_WIDTH, Point::new(CHAR_WORLD_X, CHAR_WORLD_Y)),
        }
    }

    fn draw_window(&self, renderer: &mut Renderer) {
        let mg_offset = self.base.environment.camera_offset(Layer::Midground);
        let win_sx = WINDOW_WORLD_X - mg_offset;
        let wall_bottom = WINDOW_Y + WINDOW_H;
        let screen_left = win_sx.max(0);
        let screen_right = (win_sx + WINDOW_W).min(DISPLAY_WIDTH);

        // Mask everything outside the window opening with black. Tall sprites
        // (balloon, plane) that extend below the window get covered by the
        // side and bottom rects.
        if screen_left > 0 {
            renderer.fill_rect_off(
                Point::new(0, 0),
                Size::new(screen_left as u32, DISPLAY_HEIGHT as u32),
            );
        }
        if screen_right < DISPLAY_WIDTH {
            renderer.fill_rect_off(
                Point::new(screen_right, 0),
                Size::new(
                    (DISPLAY_WIDTH - screen_right) as u32,
                    DISPLAY_HEIGHT as u32,
                ),
            );
        }
        if WINDOW_Y > 0 {
            renderer.fill_rect_off(
                Point::new(screen_left, 0),
                Size::new((screen_right - screen_left) as u32, WINDOW_Y as u32),
            );
        }
        if wall_bottom < DISPLAY_HEIGHT && screen_right > screen_left {
            renderer.fill_rect_off(
                Point::new(screen_left, wall_bottom),
                Size::new(
                    (screen_right - screen_left) as u32,
                    (DISPLAY_HEIGHT - wall_bottom) as u32,
                ),
            );
        }

        // Window frame: inner outline, outer outline, and a filled sill.
        renderer.draw_rect(
            Point::new(win_sx, WINDOW_Y),
            Size::new(WINDOW_W as u32, WINDOW_H as u32),
            false,
        );
        renderer.draw_rect(
            Point::new(win_sx - 4, WINDOW_Y - 4),
            Size::new((WINDOW_W + 8) as u32, (WINDOW_H + 8) as u32),
            false,
        );
        renderer.draw_rect(
            Point::new(win_sx - 6, WINDOW_Y + WINDOW_H + 4),
            Size::new((WINDOW_W + 12) as u32, 3),
            true,
        );
    }
}

impl Scene for InsideScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.base.enter(ctx);
        let bookshelf_y = 63 - BOOKSHELF.height as i32;
        self.base.environment.add_object(
            Layer::Foreground,
            &BOOKSHELF,
            0,
            bookshelf_y,
            false,
        );
        // TODO: BOX_SMALL_1 on top of the bookshelf, ClockWidget at world_x=36.
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
        if buttons.was_just_pressed(Button::Menu2) {
            self.base.character.skip_behavior(ctx);
        }
        // TODO: weather-change detection (Python re-enters scene when weather changes
        //       so clouds/precipitation rebuild — needed once weather affects the indoor sky).
        // TODO: BOX_SMALL_1 on top of the bookshelf (foreground sprite).
        // TODO: ClockWidget at world_x=36, world_y=0 (midground custom draw).
        // TODO: first-impression behavior trigger on first enter (Python `_first_impression_behavior`).
        // TODO: on_post_draw lightning inversion — Python explicitly calls renderer.invert() in
        //       on_post_draw so the whole room flashes. Our set_invert from draw_sky already
        //       handles it via the window, but verify it propagates correctly across all rooms.
        // TODO: character.set_pose("sitting.forward.neutral") on enter (Python overrides
        //       behavior pose). Currently the behavior owns the pose.
        // TODO: plant surfaces (PLANT_SURFACES) once the plant system is ported.
        None
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        self.base.draw_sky(renderer, ctx);
        self.draw_window(renderer);
        self.base.draw_layers(renderer);
        self.base.draw_character(renderer);
        self.base.draw_dev_overlay(renderer, ctx);
    }
}
