use embedded_graphics::prelude::Point;

use crate::{
    behavior::{BehaviorManager, EatingSource, NextBehavior},
    context::GameContext,
    entities::character::Character,
    environment::{Environment, Layer},
    input::{Button, Buttons},
    render::Renderer,
    scene::SceneId,
    sky::SkyRenderer,
    ui::{
        burst::BurstEffect,
        location_menu::{LocationAction, LocationMenu, LocationMenuResult},
    },
};

const PAN_SPEED: i32 = 4;

pub struct LocationScene {
    pub environment: Environment,
    pub character: Character,
    pub sky: SkyRenderer,
    pub behaviors: BehaviorManager,
    menu: LocationMenu,
    menu_active: bool,
    burst: BurstEffect,
}

impl LocationScene {
    pub fn new(world_width: i32, character_pos: Point) -> Self {
        Self {
            environment: Environment::new(world_width),
            character: Character::new(character_pos),
            sky: SkyRenderer::new(world_width),
            behaviors: BehaviorManager::new(),
            menu: LocationMenu::new(),
            menu_active: false,
            burst: BurstEffect::new(),
        }
    }

    pub fn menu_active(&self) -> bool {
        self.menu_active
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
        // Capture per-frame input + camera so behaviors can observe both
        // without taking Buttons / Environment refs (avoids plumbing them
        // through the Behavior trait).
        ctx.scene_camera_x = self.environment.camera_offset(Layer::Foreground);
        ctx.input.left = buttons.is_pressed(Button::Left);
        ctx.input.right = buttons.is_pressed(Button::Right);
        ctx.input.up = buttons.is_pressed(Button::Up);
        ctx.input.down = buttons.is_pressed(Button::Down);
        ctx.input.a = buttons.is_pressed(Button::A);
        ctx.input.b = buttons.is_pressed(Button::B);
        ctx.input.a_just_pressed = !self.menu_active && buttons.was_just_pressed(Button::A);
        ctx.input.b_just_pressed = !self.menu_active && buttons.was_just_pressed(Button::B);

        // World ticks regardless of menu state — matches Python's MainScene.
        self.sky.update(ctx, dt);
        self.behaviors.update(ctx, &mut self.character, dt);
        let pose = self.behaviors.current_pose();
        self.character.set_pose(pose);
        self.character.animate(dt);
        self.character.eye_override = self.behaviors.current_eye_frame_override();
        self.burst.update(dt);

        if self.menu_active {
            return self.handle_menu_input(ctx, buttons);
        }

        if buttons.was_just_pressed(Button::Menu1) {
            return Some(SceneId::Menu);
        }
        if buttons.was_just_pressed(Button::Menu2) {
            self.menu.open(ctx);
            self.menu_active = true;
            return None;
        }

        // Suppress pan while the player is steering an active toy.
        if !self.behaviors.current_captures_dpad() {
            if buttons.is_pressed(Button::Left) {
                self.environment.pan(-PAN_SPEED);
            }
            if buttons.is_pressed(Button::Right) {
                self.environment.pan(PAN_SPEED);
            }
        }

        // Behaviors may set a pending_scene (e.g. go_to triggers a transition).
        if let Some(next) = ctx.pending_scene.take() {
            return Some(next);
        }
        None
    }

    fn handle_menu_input(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
    ) -> Option<SceneId> {
        match self.menu.handle_input(ctx, buttons) {
            LocationMenuResult::Continue => None,
            LocationMenuResult::Closed => {
                self.menu_active = false;
                None
            }
            LocationMenuResult::Action(action) => {
                self.menu_active = false;
                self.apply_menu_action(ctx, action)
            }
        }
    }

    fn apply_menu_action(
        &mut self,
        ctx: &mut GameContext,
        action: LocationAction,
    ) -> Option<SceneId> {
        match action {
            LocationAction::Affection(variant) => {
                self.behaviors.trigger(
                    NextBehavior::Affection(variant),
                    ctx,
                    &mut self.character,
                );
            }
            LocationAction::Attention(variant) => {
                self.behaviors.trigger(
                    NextBehavior::Attention(variant),
                    ctx,
                    &mut self.character,
                );
            }
            LocationAction::Eat(item) => {
                // Decrement at trigger time; mirrors Python's behaviour even
                // when the cat ultimately refuses to eat.
                let idx = item as usize;
                if ctx.food_stock[idx] > 0 {
                    ctx.food_stock[idx] -= 1;
                }
                // Face the cat inward so the food bowl appears on screen
                // (mirrors `_orient_for_eating` in main_scene.py).
                let screen_x = self.character.pos.x - self.environment.camera_x;
                self.character.mirror_h = screen_x < 64;
                self.behaviors.trigger(
                    NextBehavior::Eating(EatingSource::Item(item)),
                    ctx,
                    &mut self.character,
                );
            }
            LocationAction::Groom => {
                self.behaviors
                    .trigger(NextBehavior::BeingGroomed, ctx, &mut self.character);
            }
            LocationAction::Train(kind) => {
                self.behaviors.trigger(
                    NextBehavior::Training(kind),
                    ctx,
                    &mut self.character,
                );
            }
            LocationAction::Play(variant) => {
                self.behaviors.trigger(
                    NextBehavior::Playing(variant),
                    ctx,
                    &mut self.character,
                );
            }
            LocationAction::Medicine => {
                if ctx.medicine > 0 {
                    let first_dose = !ctx.medicine_pending;
                    ctx.medicine -= 1;
                    ctx.medicine_pending = true;
                    if first_dose {
                        self.burst.trigger_heal(&mut ctx.rng, 10);
                    }
                }
                // TODO(medicine): wire ctx.medicine_pending into the sleeping
                // / sickness recovery tick when the heal-during-rest loop is
                // ported.
            }
            LocationAction::GoToStore => return Some(SceneId::Store),
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
        self.burst.draw(renderer, screen);
    }

    pub fn draw_menu(&self, renderer: &mut Renderer) {
        self.menu.draw(renderer);
    }
}
