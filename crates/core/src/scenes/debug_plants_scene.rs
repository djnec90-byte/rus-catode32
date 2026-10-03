//! Debug viewer for plant and pot sprites. Mirrors `debug_plants.py`.
//!
//! Up/down cycles growth stage; left/right cycles health (healthy/wilted/dead).
//! Menu2 swaps between pot/plant submenus. B returns to the last main scene.

use embedded_graphics::prelude::Point;
use crate::t;

use crate::{
    assets::plants::{plant_sprite, pot_sprite, PlantStage, PotKind},
    context::{GameContext, SeedKind},
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
    scene::{Scene, SceneId},
    ui::menu::{Menu, MenuItem, MenuResult},
};

#[derive(Clone, Copy)]
enum DebugPlantsAction {
    PickPot(u8),
    PickSeed(u8),
}

const FLOOR_Y: i32 = 63;

const STAGES: &[PlantStage] = &[
    PlantStage::Seedling,
    PlantStage::Young,
    PlantStage::Growing,
    PlantStage::Mature,
    PlantStage::Thriving,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Health {
    Healthy,
    Wilted,
    Dead,
}

const HEALTHS: &[Health] = &[Health::Healthy, Health::Wilted, Health::Dead];

const POTS: &[PotKind] = &[PotKind::Small, PotKind::Medium, PotKind::Large, PotKind::Planter];
const SEEDS: &[SeedKind] = &[
    SeedKind::CatGrass,
    SeedKind::Freesia,
    SeedKind::Rose,
    SeedKind::Sunflower,
];

const POT_MENU: &[MenuItem<DebugPlantsAction>] = &[
    pot_menu_item(t!("Small"), 0),
    pot_menu_item(t!("Medium"), 1),
    pot_menu_item(t!("Large"), 2),
    pot_menu_item(t!("Planter"), 3),
];

const SEED_MENU: &[MenuItem<DebugPlantsAction>] = &[
    seed_menu_item(t!("Cat Grass"), 0),
    seed_menu_item(t!("Freesia"), 1),
    seed_menu_item(t!("Rose"), 2),
    seed_menu_item(t!("Sunflower"), 3),
];

const MENU: &[MenuItem<DebugPlantsAction>] = &[
    MenuItem {
        label: t!("Pot type"),
        icon: None,
        submenu: Some(POT_MENU),
        action: None,
        confirm: None,
        confirm_on_vacation: None,
    },
    MenuItem {
        label: t!("Plant type"),
        icon: None,
        submenu: Some(SEED_MENU),
        action: None,
        confirm: None,
        confirm_on_vacation: None,
    },
];

const fn pot_menu_item(label: &'static str, idx: u8) -> MenuItem<DebugPlantsAction> {
    MenuItem {
        label,
        icon: None,
        submenu: None,
        action: Some(DebugPlantsAction::PickPot(idx)),
        confirm: None,
        confirm_on_vacation: None,
    }
}

const fn seed_menu_item(label: &'static str, idx: u8) -> MenuItem<DebugPlantsAction> {
    MenuItem {
        label,
        icon: None,
        submenu: None,
        action: Some(DebugPlantsAction::PickSeed(idx)),
        confirm: None,
        confirm_on_vacation: None,
    }
}

pub struct DebugPlantsScene {
    pot_idx: usize,
    plant_idx: usize,
    stage_idx: usize,
    health_idx: usize,
    menu: Menu<DebugPlantsAction>,
    menu_active: bool,
}

impl DebugPlantsScene {
    pub fn new() -> Self {
        Self {
            pot_idx: 0,
            plant_idx: 0,
            stage_idx: 0,
            health_idx: 0,
            menu: Menu::new(MENU),
            menu_active: false,
        }
    }

    fn current_stage_key(&self) -> PlantStage {
        let base = STAGES[self.stage_idx];
        match HEALTHS[self.health_idx] {
            Health::Healthy => base,
            Health::Wilted => base.wilted_variant(),
            Health::Dead => base.dead_variant(),
        }
    }

    fn draw_floor(&self, renderer: &mut Renderer) {
        renderer.draw_line(Point::new(0, FLOOR_Y), Point::new(128, FLOOR_Y));
    }

    fn draw_sprites(&self, renderer: &mut Renderer) {
        let pot = POTS[self.pot_idx];
        let plant_type = SEEDS[self.plant_idx];
        let stage = self.current_stage_key();

        let cx = 64;
        let pot_h = pot_sprite(pot).map(|s| s.height as i32).unwrap_or(0);
        let pot_w = pot_sprite(pot).map(|s| s.width as i32).unwrap_or(0);
        let pot_y = FLOOR_Y - pot_h;
        if let Some(spr) = pot_sprite(pot) {
            renderer.draw_sprite(
                spr,
                Point::new(cx - pot_w / 2, pot_y),
                SpriteOpts::default(),
            );
        }
        if let Some(spr) = plant_sprite(plant_type, stage) {
            let plant_x = cx - (spr.width as i32) / 2;
            let plant_y = pot_y - spr.height as i32;
            renderer.draw_sprite(spr, Point::new(plant_x, plant_y), SpriteOpts::default());
        }
    }

    fn draw_labels(&self, renderer: &mut Renderer) {
        let stage_label = STAGES[self.stage_idx].label();
        let health_label = match HEALTHS[self.health_idx] {
            Health::Healthy => "healthy",
            Health::Wilted => "wilted",
            Health::Dead => "dead",
        };
        let seed_label = match SEEDS[self.plant_idx] {
            SeedKind::CatGrass => "cat_grass",
            SeedKind::Freesia => "freesia",
            SeedKind::Rose => "rose",
            SeedKind::Sunflower => "sunflower",
        };
        let pot_label = match POTS[self.pot_idx] {
            PotKind::Small => "small",
            PotKind::Medium => "medium",
            PotKind::Large => "large",
            PotKind::Planter => "planter",
            PotKind::Ground => "ground",
        };
        // Render two rows with simple ASCII concat (fits 128px easily).
        use heapless::String;
        let mut row1: String<32> = String::new();
        let _ = row1.push_str(seed_label);
        let _ = row1.push_str(" ");
        let _ = row1.push_str(pot_label);
        renderer.draw_text(row1.as_str(), Point::new(1, 0));

        let mut row2: String<32> = String::new();
        let _ = row2.push_str(stage_label);
        let _ = row2.push_str(" ");
        let _ = row2.push_str(health_label);
        renderer.draw_text(row2.as_str(), Point::new(1, 8));
    }

    fn apply_menu_action(&mut self, action: DebugPlantsAction) {
        match action {
            DebugPlantsAction::PickPot(idx) => {
                self.pot_idx = (idx as usize) % POTS.len();
            }
            DebugPlantsAction::PickSeed(idx) => {
                self.plant_idx = (idx as usize) % SEEDS.len();
            }
        }
    }
}

impl Scene for DebugPlantsScene {
    fn enter(&mut self, _ctx: &mut GameContext) {
        self.menu.reset_to(MENU);
        self.menu_active = false;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        if self.menu_active {
            match self.menu.handle_input(buttons, false) {
                MenuResult::Continue => return None,
                MenuResult::Closed => {
                    self.menu_active = false;
                    return None;
                }
                MenuResult::Action(action) => {
                    self.menu_active = false;
                    self.apply_menu_action(action);
                    return None;
                }
            }
        }

        if buttons.was_just_pressed(Button::B) {
            return Some(ctx.last_main_scene);
        }
        if buttons.was_just_pressed(Button::Menu2) {
            self.menu_active = true;
            self.menu.reset_to(MENU);
            return None;
        }
        if buttons.was_just_pressed(Button::Up) {
            self.stage_idx = (self.stage_idx + 1) % STAGES.len();
        } else if buttons.was_just_pressed(Button::Down) {
            self.stage_idx = (self.stage_idx + STAGES.len() - 1) % STAGES.len();
        }
        if buttons.was_just_pressed(Button::Left) {
            self.health_idx = (self.health_idx + HEALTHS.len() - 1) % HEALTHS.len();
        } else if buttons.was_just_pressed(Button::Right) {
            self.health_idx = (self.health_idx + 1) % HEALTHS.len();
        }
        None
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        if self.menu_active {
            self.menu.draw(renderer);
            return;
        }
        self.draw_floor(renderer);
        self.draw_sprites(renderer);
        self.draw_labels(renderer);
    }
}
