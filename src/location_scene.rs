use embedded_graphics::prelude::Point;

use crate::{
    behavior::{BehaviorManager, EatingSource, NextBehavior},
    context::GameContext,
    entities::character::Character,
    environment::{Environment, Layer},
    gardening_ui::{
        PlacementKind, PlacementMode, PlacementResult, PlantBursts, PlantSelectionMode,
        PlantSurface, SelectionFilter, SelectionResult,
    },
    input::{Button, Buttons},
    plant_renderer::draw_plants_layer,
    plant_system::{
        self, fertilize_plant, get_plant, get_plant_mut, move_plant, place_empty_pot,
        plant_in_ground, plant_seed_into_pot, remove_plant, repot_plant, scene_plant_health_score,
        tick_plants, water_plant, PlantLayer,
    },
    render::Renderer,
    scene::SceneId,
    sky::SkyRenderer,
    ui::{
        burst::BurstEffect,
        location_menu::{GardeningAction, LocationAction, LocationMenu, LocationMenuResult},
        popup::Popup,
    },
};

const PAN_SPEED: i32 = 4;
const DISPLAY_WIDTH: i32 = 128;
const FOLLOW_MARGIN: i32 = 32;

pub struct LocationScene {
    pub environment: Environment,
    pub character: Character,
    pub sky: SkyRenderer,
    pub behaviors: BehaviorManager,
    menu: LocationMenu,
    menu_active: bool,
    burst: BurstEffect,
    placement: PlacementMode,
    selection: PlantSelectionMode,
    plant_bursts: PlantBursts,
    scene_id: SceneId,
    plant_surfaces: &'static [PlantSurface],
    /// True while the player is chaining tend actions (Water/Fertilize) — the
    /// menu re-opens the selection cursor after each one so multiple plants
    /// can be tended in succession without re-navigating from Menu2.
    in_tend_mode: bool,
    last_tended_plant_id: Option<u32>,
    /// Pending seed kind for an in-ground placement flow. Set when the player
    /// picks "Plant Seed → In Ground → Rose" before placement starts.
    pending_ground_seed: Option<crate::context::SeedKind>,
    popup: Popup,
    popup_active: bool,
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
            placement: PlacementMode::new(),
            selection: PlantSelectionMode::new(),
            plant_bursts: PlantBursts::new(),
            scene_id: SceneId::Inside,
            plant_surfaces: &[],
            in_tend_mode: false,
            last_tended_plant_id: None,
            pending_ground_seed: None,
            popup: Popup::new(14, 12, 100, 40),
            popup_active: false,
        }
    }

    fn show_popup(&mut self, text: &str) {
        self.popup.set_text(text, true, true);
        self.popup_active = true;
    }

    pub fn menu_active(&self) -> bool {
        self.menu_active
    }

    pub fn plant_overlay_active(&self) -> bool {
        self.placement.active() || self.selection.active()
    }

    /// Per-scene entry point. `plant_surfaces` is the slice of placement
    /// surfaces defined as a const on the scene type (empty slice for scenes
    /// that don't support gardening).
    pub fn enter(
        &mut self,
        ctx: &mut GameContext,
        scene_id: SceneId,
        plant_surfaces: &'static [PlantSurface],
    ) {
        ctx.last_main_scene = scene_id;
        self.scene_id = scene_id;
        self.plant_surfaces = plant_surfaces;
        // TODO(scene_bounds): pull these from per-scene constants; today every
        // scene shares the default character walkable strip.
        ctx.scene_x_min = 10;
        ctx.scene_x_max = (self.environment.world_width - 10).max(10);
        self.character.reseed_anim();
        self.environment
            .set_camera(self.character.pos.x - DISPLAY_WIDTH / 2);
        self.behaviors.start(ctx, &mut self.character);
        self.placement.cancel();
        self.selection.cancel();
        self.in_tend_mode = false;
        self.pending_ground_seed = None;

        // Honor a pending cross-scene plant move arriving at this scene.
        if let Some(pending) = ctx.pending_gardening_move {
            if pending.dest_scene == scene_id && !plant_surfaces.is_empty() {
                if let Some(plant) = get_plant(ctx, pending.plant_id) {
                    let pot = plant.pot;
                    let id = plant.id;
                    if pot != crate::assets::plants::PotKind::Ground {
                        self.placement.enter(
                            PlacementKind::Move { pot, plant_id: id },
                            plant_surfaces,
                            &self.environment,
                        );
                    }
                }
                ctx.pending_gardening_move = None;
            }
        }
    }

    pub fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        ctx.scene_camera_x = self.environment.camera_offset(Layer::Foreground);
        ctx.input.left = buttons.is_pressed(Button::Left);
        ctx.input.right = buttons.is_pressed(Button::Right);
        ctx.input.up = buttons.is_pressed(Button::Up);
        ctx.input.down = buttons.is_pressed(Button::Down);
        ctx.input.a = buttons.is_pressed(Button::A);
        ctx.input.b = buttons.is_pressed(Button::B);
        // `was_just_pressed` is consume-once: calling it here marks the edge
        // as seen and prevents downstream readers (menu, placement, selection)
        // from observing the same press. Only sample for behaviors when there
        // is no overlay competing for the press.
        let overlay_active = self.menu_active
            || self.placement.active()
            || self.selection.active()
            || self.popup_active;
        ctx.input.a_just_pressed = !overlay_active && buttons.was_just_pressed(Button::A);
        ctx.input.b_just_pressed = !overlay_active && buttons.was_just_pressed(Button::B);

        // Refresh the aggregate plant-health score behaviors read from ctx.
        let score = scene_plant_health_score(ctx, self.scene_id).clamp(-100, 100);
        ctx.scene_plant_health = score as i8;

        // World ticks regardless of menu state — matches Python's MainScene.
        self.sky.update(ctx, dt);
        // Advance plants once per in-game hour (catches up if many hours elapsed).
        tick_plants(ctx);
        let prev_x = self.character.pos.x;
        self.behaviors.update(ctx, &mut self.character, dt);
        let pose = self.behaviors.current_pose();
        self.character.set_pose(pose);
        self.character.animate(dt);
        self.character.eye_override = self.behaviors.current_eye_frame_override();
        self.burst.update(dt);
        self.plant_bursts.update(dt);
        if self.placement.active() {
            self.placement.update(dt);
        }
        if self.selection.active() {
            self.selection.update(dt);
        }

        // Auto-follow: when the cat moves on its own, nudge the camera so it
        // stays within FOLLOW_MARGIN of the screen edges. Suppressed while the
        // player is actively panning or steering plant cursors.
        let panning = buttons.is_pressed(Button::Left) || buttons.is_pressed(Button::Right);
        let cursor_active = self.placement.active() || self.selection.active();
        if !panning && !cursor_active && self.character.pos.x != prev_x {
            let screen_x = self.character.pos.x - self.environment.camera_x;
            if screen_x < FOLLOW_MARGIN {
                self.environment
                    .set_camera(self.character.pos.x - FOLLOW_MARGIN);
            } else if screen_x > DISPLAY_WIDTH - FOLLOW_MARGIN {
                self.environment
                    .set_camera(self.character.pos.x - (DISPLAY_WIDTH - FOLLOW_MARGIN));
            }
        }

        // Popup messages are modal: A or B dismisses, and if the player was
        // tending we re-enter the selection cursor for the next action.
        if self.popup_active {
            if buttons.was_just_pressed(Button::A)
                || buttons.was_just_pressed(Button::B)
                || buttons.was_just_pressed(Button::Menu1)
                || buttons.was_just_pressed(Button::Menu2)
            {
                self.popup_active = false;
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
            } else if buttons.was_just_pressed(Button::Down) {
                self.popup.scroll_down();
            } else if buttons.was_just_pressed(Button::Up) {
                self.popup.scroll_up();
            }
            return None;
        }

        // Cursor modes take precedence over the menu so the player can confirm
        // placement / selection without first dismissing the menu.
        if self.placement.active() {
            return self.handle_placement_input(ctx, buttons);
        }
        if self.selection.active() {
            return self.handle_selection_input(ctx, buttons);
        }

        if self.menu_active {
            return self.handle_menu_input(ctx, buttons);
        }

        if buttons.was_just_pressed(Button::Menu1) {
            return Some(SceneId::Menu);
        }
        if buttons.was_just_pressed(Button::Menu2) {
            self.menu
                .open(ctx, self.scene_id, !self.plant_surfaces.is_empty());
            self.menu_active = true;
            return None;
        }

        if !self.behaviors.current_captures_dpad() {
            if buttons.is_pressed(Button::Left) {
                self.environment.pan(-PAN_SPEED);
            }
            if buttons.is_pressed(Button::Right) {
                self.environment.pan(PAN_SPEED);
            }
        }

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
                self.in_tend_mode = false;
                None
            }
            LocationMenuResult::Action(action) => {
                self.menu_active = false;
                self.apply_menu_action(ctx, action)
            }
        }
    }

    fn handle_placement_input(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
    ) -> Option<SceneId> {
        let kind = self.placement.current_kind();
        match self.placement.handle_input(buttons, &mut self.environment) {
            PlacementResult::Continue => None,
            PlacementResult::Cancelled => {
                self.pending_ground_seed = None;
                // Re-enter tend selection if we cancelled out of a within-scene
                // move; otherwise just drop back to play.
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
                None
            }
            PlacementResult::Confirm { layer, x, y_snap } => {
                self.commit_placement(ctx, kind, layer, x, y_snap);
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
                None
            }
        }
    }

    fn commit_placement(
        &mut self,
        ctx: &mut GameContext,
        kind: PlacementKind,
        layer: PlantLayer,
        x: i32,
        y_snap: i32,
    ) {
        match kind {
            PlacementKind::Pot(pot) => {
                // Cap to 16 plants per scene (matches Python design memo).
                let scene_count = ctx.plants.iter().filter(|p| p.scene == self.scene_id).count();
                if scene_count < 16 {
                    let _ = place_empty_pot(ctx, self.scene_id, layer, x, y_snap, pot);
                }
            }
            PlacementKind::Ground => {
                if let Some(seed) = self.pending_ground_seed.take() {
                    let scene_count =
                        ctx.plants.iter().filter(|p| p.scene == self.scene_id).count();
                    if scene_count < 16 {
                        let _ = plant_in_ground(ctx, self.scene_id, layer, x, y_snap, seed);
                    }
                }
            }
            PlacementKind::GroundSeed => {
                if let Some(seed) = self.pending_ground_seed.take() {
                    let scene_count =
                        ctx.plants.iter().filter(|p| p.scene == self.scene_id).count();
                    if scene_count < 16 {
                        let _ = plant_in_ground(ctx, self.scene_id, layer, x, y_snap, seed);
                    }
                }
            }
            PlacementKind::Move { plant_id, .. } => {
                move_plant(ctx, plant_id, self.scene_id, layer, x, y_snap);
            }
        }
    }

    fn handle_selection_input(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
    ) -> Option<SceneId> {
        match self.selection.handle_input(ctx, buttons, &mut self.environment) {
            SelectionResult::Continue => None,
            SelectionResult::Cancelled => {
                self.in_tend_mode = false;
                self.pending_ground_seed = None;
                None
            }
            SelectionResult::Confirm(id) => {
                self.handle_selection_confirm(ctx, id);
                None
            }
        }
    }

    fn handle_selection_confirm(&mut self, ctx: &mut GameContext, id: u32) {
        // Selection is reused for two flows: tending an existing plant, and
        // dropping a seed into an empty pot. Inspect which mode we're in by
        // looking at whether a pending seed was queued.
        if let Some(seed) = self.pending_ground_seed.take() {
            // We were planting a seed; this id is the empty pot.
            let _ = plant_seed_into_pot(ctx, id, seed);
            return;
        }
        // Otherwise: open the dynamic Tend menu for this plant.
        self.last_tended_plant_id = Some(id);
        self.in_tend_mode = true;
        self.menu.open_tend(ctx, self.scene_id, id);
        self.menu_active = true;
    }

    /// Re-open the plant-selection cursor for the next tend action. Drops out
    /// of tend mode if no plants remain in the scene.
    fn reenter_tend_selection(&mut self, ctx: &GameContext) {
        let found = self.selection.enter(
            ctx,
            self.scene_id,
            SelectionFilter::All,
            self.last_tended_plant_id,
        );
        if !found {
            self.in_tend_mode = false;
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
                let idx = item as usize;
                if ctx.food_stock[idx] > 0 {
                    ctx.food_stock[idx] -= 1;
                }
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
            }
            LocationAction::GoToStore => return Some(SceneId::Store),
            LocationAction::Gardening(action) => return self.apply_gardening_action(ctx, action),
        }
        None
    }

    fn apply_gardening_action(
        &mut self,
        ctx: &mut GameContext,
        action: GardeningAction,
    ) -> Option<SceneId> {
        match action {
            GardeningAction::PlacePot(pot) => {
                self.placement.enter(
                    PlacementKind::Pot(pot),
                    self.plant_surfaces,
                    &self.environment,
                );
            }
            GardeningAction::PlantSeedInPot(seed) => {
                if !ctx.owns_tool(crate::context::ToolKind::Spade) {
                    self.show_popup("You need the Spade to plant seeds. Buy one at the store!");
                    return None;
                }
                self.pending_ground_seed = Some(seed);
                let found =
                    self.selection
                        .enter(ctx, self.scene_id, SelectionFilter::EmptyPot, None);
                if !found {
                    self.pending_ground_seed = None;
                    self.show_popup("No empty pots in this location");
                }
            }
            GardeningAction::PlantSeedInGround(seed) => {
                if !ctx.owns_tool(crate::context::ToolKind::Spade) {
                    self.show_popup("You need the Spade to plant seeds. Buy one at the store!");
                    return None;
                }
                self.pending_ground_seed = Some(seed);
                self.placement.enter(
                    PlacementKind::Ground,
                    self.plant_surfaces,
                    &self.environment,
                );
            }
            GardeningAction::StartTend => {
                self.in_tend_mode = true;
                let found = self.selection.enter(
                    ctx,
                    self.scene_id,
                    SelectionFilter::All,
                    self.last_tended_plant_id,
                );
                if !found {
                    self.in_tend_mode = false;
                }
            }
            GardeningAction::Water(id) => {
                if !ctx.owns_tool(crate::context::ToolKind::WateringCan) {
                    self.show_popup(
                        "You need the Watering Can to water plants. Buy one at the store!",
                    );
                    return None;
                }
                if let Some(p) = get_plant_mut(ctx, id) {
                    water_plant(p);
                    let mut rng = ctx.rng;
                    self.plant_bursts.trigger(id, &mut rng, 4);
                    ctx.rng = rng;
                }
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
            }
            GardeningAction::Fertilize(id) => {
                if ctx.fertilizer > 0 {
                    if let Some(p) = get_plant_mut(ctx, id) {
                        fertilize_plant(p);
                        ctx.fertilizer = ctx.fertilizer.saturating_sub(1);
                        let mut rng = ctx.rng;
                        self.plant_bursts.trigger(id, &mut rng, 4);
                        ctx.rng = rng;
                    }
                }
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
            }
            GardeningAction::Pluck(id) => {
                remove_plant(ctx, id);
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
            }
            GardeningAction::Repot(id, target) => {
                repot_plant(ctx, id, target);
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
            }
            GardeningAction::MoveHere(id) => {
                if let Some(plant) = get_plant(ctx, id) {
                    let pot = plant.pot;
                    if pot != crate::assets::plants::PotKind::Ground {
                        self.placement.enter(
                            PlacementKind::Move { pot, plant_id: id },
                            self.plant_surfaces,
                            &self.environment,
                        );
                    }
                }
            }
            GardeningAction::MoveTo(id, dest) => {
                ctx.pending_gardening_move = Some(crate::context::PendingGardeningMove {
                    plant_id: id,
                    dest_scene: dest,
                });
                self.in_tend_mode = false;
                return Some(dest);
            }
            GardeningAction::InspectDismiss => {
                if self.in_tend_mode {
                    self.reenter_tend_selection(ctx);
                }
            }
        }
        None
    }

    pub fn draw_sky(&self, renderer: &mut Renderer, ctx: &GameContext) {
        renderer.set_invert(self.sky.lightning_invert());
        self.sky.draw(renderer, ctx, self.environment.camera_x);
    }

    pub fn draw_layers(&self, renderer: &mut Renderer) {
        self.environment.draw_all_layers(renderer);
    }

    pub fn draw_plants(&self, ctx: &GameContext, renderer: &mut Renderer, layer: PlantLayer) {
        if self.plant_surfaces.is_empty() {
            return;
        }
        draw_plants_layer(ctx, renderer, &self.environment, self.scene_id, layer);
    }

    pub fn draw_character(&self, renderer: &mut Renderer, ctx: &GameContext) {
        let camera_offset = self.environment.camera_offset(Layer::Foreground);
        self.character.draw(renderer, camera_offset);
        let screen = Point::new(
            self.character.pos.x - camera_offset,
            self.character.pos.y,
        );
        self.behaviors
            .draw_overlay(renderer, ctx, screen, self.character.mirror_h);
        self.burst.draw(renderer, screen);
        self.plant_bursts.draw(ctx, renderer, &self.environment);
    }

    pub fn draw_menu(&self, renderer: &mut Renderer) {
        self.menu.draw(renderer);
    }

    pub fn draw_overlay(&self, ctx: &GameContext, renderer: &mut Renderer) {
        // Cursor overlays (placement reticle, selection marker). Drawn last so
        // they sit above sprites and plants.
        self.placement.draw(renderer, &self.environment);
        self.selection.draw(ctx, renderer, &self.environment);
        if self.popup_active {
            self.popup.draw(renderer, true);
        }
    }
}

// Reuse plant_system helpers in other modules via re-export.
#[allow(unused_imports)]
pub use plant_system::scene_plant_health_score as plant_health;
