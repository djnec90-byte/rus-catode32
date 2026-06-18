use core::fmt::Write;

use embedded_graphics::prelude::Point;
use heapless::String;

use crate::{
    behavior::BehaviorManager,
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
    pub behaviors: BehaviorManager,
}

impl LocationScene {
    pub fn new(world_width: i32, character_pos: Point) -> Self {
        Self {
            environment: Environment::new(world_width),
            character: Character::new(character_pos),
            sky: SkyRenderer::new(world_width),
            behaviors: BehaviorManager::new(),
        }
    }

    pub fn enter(&mut self, ctx: &mut GameContext, scene_id: SceneId) {
        ctx.last_main_scene = scene_id;
        // TODO(scene_bounds): pull these from per-scene constants; today every
        // scene shares the default character walkable strip.
        ctx.scene_x_min = 10;
        ctx.scene_x_max = (self.environment.world_width - 10).max(10);
        self.character.reseed_anim();
        self.behaviors.start(ctx, &mut self.character);
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

        if buttons.is_pressed(Button::Left) {
            self.environment.pan(-PAN_SPEED);
        }
        if buttons.is_pressed(Button::Right) {
            self.environment.pan(PAN_SPEED);
        }

        self.sky.update(ctx, dt);
        self.behaviors.update(ctx, &mut self.character, dt);
        let pose = self.behaviors.current_pose();
        self.character.set_pose(pose);
        self.character.animate(dt);

        // Behaviors may set a pending_scene (e.g. go_to triggers a transition).
        if let Some(next) = ctx.pending_scene.take() {
            return Some(next);
        }
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

    pub fn draw_character(&self, renderer: &mut Renderer, ctx: &GameContext) {
        let camera_offset = self.environment.camera_offset(Layer::Foreground);
        self.character.draw(renderer, camera_offset);
        // Behavior overlay (Z's, bubbles, particle effects).
        let screen = Point::new(
            self.character.pos.x - camera_offset,
            self.character.pos.y,
        );
        self.behaviors
            .draw_overlay(renderer, ctx, screen, self.character.mirror_h);
    }

    // TODO: remove once StatsScene + sky/clock convey this information through real UI.
    pub fn draw_dev_overlay(&self, renderer: &mut Renderer, ctx: &GameContext) {
        let mut buf: String<24> = String::new();
        let _ = write!(
            buf,
            "{} {:02}:{:02}",
            self.behaviors.current_name(),
            ctx.time_hours,
            ctx.time_minutes,
        );
        renderer.draw_text(buf.as_str(), Point::new(0, 0));

        buf.clear();
        let _ = write!(buf, "{} {}", ctx.season.name(), ctx.weather.name());
        renderer.draw_text(buf.as_str(), Point::new(0, 10));
    }
}
