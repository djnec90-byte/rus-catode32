use core::fmt::Write;

use embedded_graphics::prelude::Point;
use heapless::String;

use crate::{
    context::GameContext,
    entities::character::Character,
    environment::{Environment, Layer},
    input::{Button, Buttons},
    render::Renderer,
    scene::SceneId,
    sky::SkyRenderer,
};

const PAN_SPEED: i32 = 4;

pub struct LocationScene {
    pub environment: Environment,
    pub character: Character,
    pub sky: SkyRenderer,
}

impl LocationScene {
    pub fn new(world_width: i32, character_pos: Point) -> Self {
        Self {
            environment: Environment::new(world_width),
            character: Character::new(character_pos),
            sky: SkyRenderer::new(world_width),
        }
    }

    pub fn enter(&mut self, ctx: &mut GameContext) {
        self.character.enter(ctx);
    }

    pub fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::Menu1) {
            return Some(SceneId::Menu);
        }
        // Menu2 binding is owned by each location scene so they can choose what
        // it does (skip_behavior, debug spawns, etc.).

        if buttons.is_pressed(Button::Left) {
            self.environment.pan(-PAN_SPEED);
        }
        if buttons.is_pressed(Button::Right) {
            self.environment.pan(PAN_SPEED);
        }

        self.sky.update(ctx, dt);
        self.character.update(ctx, dt);
        None
    }

    pub fn draw_sky(&self, renderer: &mut Renderer, ctx: &GameContext) {
        renderer.set_invert(self.sky.lightning_invert());
        self.sky.draw(renderer, ctx, self.environment.camera_x);
    }

    pub fn draw_layers(&self, renderer: &mut Renderer) {
        self.environment.draw_all_layers(renderer);
        // TODO (Stage 4): draw precipitation overlay using midground parallax.
    }

    pub fn draw_character(&self, renderer: &mut Renderer) {
        let camera_offset = self.environment.camera_offset(Layer::Foreground);
        self.character.draw(renderer, camera_offset);
    }

    // TODO: remove once StatsScene + sky/clock convey this information through real UI.
    pub fn draw_dev_overlay(&self, renderer: &mut Renderer, ctx: &GameContext) {
        let mut buf: String<24> = String::new();
        let _ = write!(
            buf,
            "{} {:02}:{:02}",
            self.character.current_behavior_name(),
            ctx.time_hours,
            ctx.time_minutes,
        );
        renderer.draw_text(buf.as_str(), Point::new(0, 0));

        buf.clear();
        let _ = write!(buf, "{} {}", ctx.season.name(), ctx.weather.name());
        renderer.draw_text(buf.as_str(), Point::new(0, 10));
    }
}
