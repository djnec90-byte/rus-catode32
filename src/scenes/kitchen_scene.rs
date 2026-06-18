use embedded_graphics::prelude::{Point, Size};

use crate::{
    assets::furniture::FAUCET,
    context::GameContext,
    environment::Layer,
    input::{Button, Buttons},
    location_scene::LocationScene,
    render::{Renderer, SpriteOpts},
    scene::{Scene, SceneId},
};

const WORLD_WIDTH: i32 = 192;
const CHAR_WORLD_X: i32 = 64;
const CHAR_WORLD_Y: i32 = 64;

const COUNTER_X_START: i32 = 25;
const COUNTER_X_END: i32 = 175;
const COUNTER_TOP_Y: i32 = 24;
const FAUCET_WORLD_X: i32 = 82;

pub struct KitchenScene {
    base: LocationScene,
}

impl KitchenScene {
    pub fn new() -> Self {
        Self {
            base: LocationScene::new(WORLD_WIDTH, Point::new(CHAR_WORLD_X, CHAR_WORLD_Y)),
        }
    }

    fn draw_counter(&self, renderer: &mut Renderer) {
        let mg_offset = self.base.environment.camera_offset(Layer::Midground);
        let sx = (COUNTER_X_START - mg_offset).max(0);
        let ex = (COUNTER_X_END - mg_offset).min(128);
        if sx >= ex {
            return;
        }

        // Counter top
        renderer.draw_rect(
            Point::new(sx - 3, COUNTER_TOP_Y),
            Size::new((ex - sx + 6) as u32, 4),
            true,
        );
        // Cabinet front outline
        renderer.draw_rect(
            Point::new(sx, COUNTER_TOP_Y + 4),
            Size::new((ex - sx) as u32, 30),
            false,
        );
        // Cabinet door dividers
        let mut door_wx = 40;
        while door_wx < COUNTER_X_END {
            let dx = door_wx - mg_offset;
            if sx < dx && dx < ex {
                renderer.draw_line(Point::new(dx, COUNTER_TOP_Y + 4), Point::new(dx, 57));
            }
            door_wx += 30;
        }

        // Faucet sits on the counter (sprite y = top_y - faucet_h)
        let fx = FAUCET_WORLD_X - mg_offset;
        let fy = COUNTER_TOP_Y - FAUCET.height as i32;
        renderer.draw_sprite(&FAUCET, Point::new(fx, fy), SpriteOpts::default());
    }
}

impl Scene for KitchenScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.base.enter(ctx, SceneId::Kitchen);
        // TODO: ClockWidget at world_x=100, world_y=0 (midground custom draw).
        // TODO: BOX_SMALL_1 and FOOD_BOWL items (Python adds them as foreground sprites).
        // TODO: character.set_pose("sitting.forward.neutral") on enter (Python override).
        // TODO: plant surfaces (PLANT_SURFACES) once the plant system is ported.
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
            self.base.behaviors.skip(ctx, &mut self.base.character);
        }
        // TODO: ClockWidget.set_time(hours, minutes) per frame.
        // TODO: on_post_draw lightning inversion for indoor rooms with no sky drawn.
        None
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        // Closed room — no sky.
        self.base.environment.draw_layer(renderer, Layer::Background);
        self.base.environment.draw_layer(renderer, Layer::Midground);
        self.draw_counter(renderer);
        self.base.environment.draw_layer(renderer, Layer::Foreground);
        self.base.draw_character(renderer, ctx);
        self.base.draw_dev_overlay(renderer, ctx);
    }
}
