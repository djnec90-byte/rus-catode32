use embedded_graphics::prelude::{Point, Size};
use crate::platform::time::Instant;

use crate::{
    assets::{
        nature::{FISH1, FISH2, FISH3, OCTOPUS, ROCK_PILE, SEAWEED, SEA_PLANT},
        plants::TINY_FLOWER,
    },
    context::{GameContext, StatId},
    entities::aquarium::{BubbleGroup, DebrisField, FishEntity, OctopusEntity},
    environment::Layer,
    location_scene::LocationScene,
    render::{Renderer, Sprite, SpriteOpts},
    scene::SceneId,
    scenes::vacation_base::{VacationConfig, VacationScene, VacationWorld},
};

const WORLD_WIDTH: i32 = 342;
const GROUND_Y: i32 = 63;
const CHAR_WORLD_X: i32 = 64;

// Tank window geometry (world coordinates, midground 0.6x parallax).
const TANK1_LEFT: i32 = 4;
const TANK1_RIGHT: i32 = 123;
const TANK2_LEFT: i32 = 132;
const TANK2_RIGHT: i32 = 251;
const TANK_TOP: i32 = 0;
const TANK_FLOOR: i32 = 50;

// Rock 1 partially behind the left wall; rock 2 straddles the divider; rock 3
// partially behind the right wall. Anchors picked so visible rock area is
// roughly even across both tanks.
const ROCK1_X: i32 = TANK1_LEFT - 14;
const ROCK2_X: i32 = TANK1_RIGHT - 19;
const ROCK3_X: i32 = TANK2_RIGHT - 33;

const SW_FRAMES: usize = 8;
const SW_FRAME_INTERVAL: f32 = 0.3;

/// Seaweed placement: world_x, y_bottom (sprite bottom), phase offset (frame).
struct SeaweedSpawn {
    x: i32,
    y_bottom: i32,
    frame_off: usize,
}

const SEAWEED_POS: &[SeaweedSpawn] = &[
    SeaweedSpawn { x: 8, y_bottom: TANK_FLOOR - ROCK_PILE_H, frame_off: 0 },
    SeaweedSpawn { x: 22, y_bottom: TANK_FLOOR - ROCK_PILE_H + 4, frame_off: 3 },
    SeaweedSpawn { x: 50, y_bottom: TANK_FLOOR, frame_off: 6 },
    SeaweedSpawn { x: 82, y_bottom: TANK_FLOOR, frame_off: 1 },
    SeaweedSpawn { x: 88, y_bottom: TANK_FLOOR, frame_off: 5 },
    SeaweedSpawn { x: 110, y_bottom: TANK_FLOOR - ROCK_PILE_H + 12, frame_off: 5 },
    SeaweedSpawn { x: 140, y_bottom: TANK_FLOOR - ROCK_PILE_H + 4, frame_off: 2 },
    SeaweedSpawn { x: 162, y_bottom: TANK_FLOOR, frame_off: 1 },
    SeaweedSpawn { x: 168, y_bottom: TANK_FLOOR, frame_off: 7 },
    SeaweedSpawn { x: 205, y_bottom: TANK_FLOOR, frame_off: 4 },
    SeaweedSpawn { x: 230, y_bottom: TANK_FLOOR - ROCK_PILE_H + 4, frame_off: 2 },
];

// Rock dimensions baked as consts since the Sprite static can't be queried in
// const context. Keep in sync with the sprite literals in assets/nature.rs.
const ROCK_PILE_H: i32 = 23;

const SEA_PLANT_POS: &[(i32, i32)] = &[
    (55, TANK_FLOOR - 11),
    (180, TANK_FLOOR - 11),
];

// Creature swim bounds. y_bottom for each sprite is computed at spawn time.
const C_X_LEFT: f32 = (TANK1_LEFT + 2) as f32;
const C_X_RIGHT: f32 = (TANK2_RIGHT - 2) as f32;
const C_Y_TOP: f32 = (TANK_TOP + 3) as f32;

struct FishSpawn {
    sprite: &'static Sprite,
    x: f32,
    y: f32,
    speed: f32,
}

const FISH_SPAWNS: &[FishSpawn] = &[
    FishSpawn { sprite: &FISH1, x: 30.0, y: 14.0, speed: 22.0 },
    FishSpawn { sprite: &FISH1, x: 190.0, y: 28.0, speed: 24.0 },
    FishSpawn { sprite: &FISH1, x: 100.0, y: 28.0, speed: 24.0 },
    FishSpawn { sprite: &FISH1, x: 220.0, y: 22.0, speed: 24.0 },
    FishSpawn { sprite: &FISH2, x: 60.0, y: 20.0, speed: 16.0 },
    FishSpawn { sprite: &FISH2, x: 270.0, y: 35.0, speed: 15.0 },
    FishSpawn { sprite: &FISH2, x: 200.0, y: 35.0, speed: 15.0 },
    FishSpawn { sprite: &FISH3, x: 100.0, y: 30.0, speed: 12.0 },
    FishSpawn { sprite: &FISH3, x: 200.0, y: 20.0, speed: 12.0 },
];

const OCT_START_X: f32 = 45.0;
const OCT_START_Y: f32 = 28.0;

const TANK_RANGES: &[(i32, i32)] = &[
    (TANK1_LEFT + 1, TANK1_RIGHT - 1),
    (TANK2_LEFT + 1, TANK2_RIGHT - 1),
];

const DEBRIS_COUNT: usize = 10;
const MAX_FISH: usize = 12;

#[derive(Default)]
pub struct AquariumWorld {
    sw_timer: f32,
    sw_frame: usize,
    fish: heapless::Vec<FishEntity, MAX_FISH>,
    octopus: Option<OctopusEntity>,
    bubbles: Option<BubbleGroup>,
    debris: Option<DebrisField>,
    rng: u32,
}

pub type VacationAquariumScene = VacationScene<AquariumWorld>;

impl AquariumWorld {
    fn spawn_creatures(&mut self) {
        self.fish.clear();
        for s in FISH_SPAWNS {
            if self.fish.is_full() {
                break;
            }
            let h = s.sprite.height as f32;
            let y_bot = (TANK_FLOOR as f32) - h - 2.0;
            let _ = self.fish.push(FishEntity::new(
                s.sprite,
                s.x,
                s.y,
                s.speed,
                C_X_LEFT,
                C_X_RIGHT,
                C_Y_TOP,
                y_bot,
                &mut self.rng,
            ));
        }
        let oct_right = C_X_RIGHT - OCTOPUS.width as f32;
        self.octopus = Some(OctopusEntity::new(
            &OCTOPUS,
            OCT_START_X,
            OCT_START_Y,
            C_X_LEFT,
            oct_right,
            &mut self.rng,
        ));
        self.bubbles = Some(BubbleGroup::new(
            &TINY_FLOWER,
            TANK_RANGES,
            TANK_TOP + 1,
            TANK_FLOOR - 1,
            &mut self.rng,
        ));
        self.debris = Some(DebrisField::new(
            DEBRIS_COUNT,
            TANK_RANGES,
            TANK_TOP + 1,
            TANK_FLOOR - 1,
            &mut self.rng,
        ));
    }

    fn draw_rocks_and_plants(&self, renderer: &mut Renderer, base: &LocationScene) {
        let offset = base.environment.camera_offset(Layer::Midground);
        let rock_w = ROCK_PILE.width as i32;
        let rock_y = TANK_FLOOR - ROCK_PILE_H;
        for &wx in &[ROCK1_X, ROCK2_X, ROCK3_X] {
            let sx = wx - offset;
            if sx + rock_w >= 0 && sx < 128 {
                renderer.draw_sprite(&ROCK_PILE, Point::new(sx, rock_y), SpriteOpts::default());
            }
        }
        let sp_w = SEA_PLANT.width as i32;
        for &(wx, sy) in SEA_PLANT_POS {
            let sx = wx - offset;
            if sx + sp_w >= 0 && sx < 128 {
                renderer.draw_sprite(&SEA_PLANT, Point::new(sx, sy), SpriteOpts::default());
            }
        }
    }

    fn draw_seaweed(&self, renderer: &mut Renderer, base: &LocationScene) {
        let offset = base.environment.camera_offset(Layer::Midground);
        let w = SEAWEED.width as i32;
        let h = SEAWEED.height as i32;
        for spawn in SEAWEED_POS {
            let sx = spawn.x - offset;
            if sx + w < 0 || sx >= 128 {
                continue;
            }
            let frame = (self.sw_frame + spawn.frame_off) % SW_FRAMES;
            renderer.draw_sprite(
                &SEAWEED,
                Point::new(sx, spawn.y_bottom - h),
                SpriteOpts {
                    frame,
                    ..Default::default()
                },
            );
        }
    }

    fn draw_creatures(&self, renderer: &mut Renderer, base: &LocationScene) {
        let offset = base.environment.camera_offset(Layer::Midground);
        if let Some(oct) = self.octopus.as_ref() {
            oct.draw(renderer, offset);
        }
        for f in &self.fish {
            f.draw(renderer, offset);
        }
    }

    fn draw_particles(&self, renderer: &mut Renderer, base: &LocationScene) {
        let offset = base.environment.camera_offset(Layer::Midground);
        if let Some(b) = self.bubbles.as_ref() {
            b.draw(renderer, offset);
        }
        if let Some(d) = self.debris.as_ref() {
            d.draw(renderer, offset);
        }
    }

    fn draw_occluders(&self, renderer: &mut Renderer, base: &LocationScene) {
        let offset = base.environment.camera_offset(Layer::Midground);
        let tank_h = (TANK_FLOOR - TANK_TOP + 1) as u32;

        let left_sx = TANK1_LEFT - offset;
        if left_sx > 0 {
            renderer.fill_rect_off(
                Point::new(0, TANK_TOP),
                Size::new(left_sx as u32, tank_h),
            );
        }

        let div_sx = TANK1_RIGHT + 1 - offset;
        let div_w = (TANK2_LEFT - TANK1_RIGHT - 1) as i32;
        if div_sx < 128 && div_sx + div_w > 0 {
            let clamped_x = div_sx.max(0);
            let clamped_w = (div_sx + div_w - clamped_x).max(0).min(128 - clamped_x);
            if clamped_w > 0 {
                renderer.fill_rect_off(
                    Point::new(clamped_x, TANK_TOP),
                    Size::new(clamped_w as u32, tank_h),
                );
            }
        }

        let right_sx = TANK2_RIGHT + 1 - offset;
        if right_sx < 128 {
            let x = right_sx.max(0);
            renderer.fill_rect_off(
                Point::new(x, TANK_TOP),
                Size::new((128 - x) as u32, tank_h),
            );
        }
    }

    fn draw_tank_outlines(&self, renderer: &mut Renderer, base: &LocationScene) {
        let offset = base.environment.camera_offset(Layer::Midground);
        let x1 = TANK1_LEFT - offset;
        renderer.draw_rect(
            Point::new(x1, TANK_TOP),
            Size::new(
                (TANK1_RIGHT - TANK1_LEFT + 1) as u32,
                (TANK_FLOOR - TANK_TOP + 1) as u32,
            ),
            false,
        );
        let x2 = TANK2_LEFT - offset;
        renderer.draw_rect(
            Point::new(x2, TANK_TOP),
            Size::new(
                (TANK2_RIGHT - TANK2_LEFT + 1) as u32,
                (TANK_FLOOR - TANK_TOP + 1) as u32,
            ),
            false,
        );
    }
}

impl VacationWorld for AquariumWorld {
    const SCENE_ID: SceneId = SceneId::VacationAquarium;
    const WORLD_WIDTH: i32 = WORLD_WIDTH;
    const CHAR_WORLD_X: i32 = CHAR_WORLD_X;
    const GROUND_Y: i32 = GROUND_Y;
    const X_MIN: i32 = 10;
    const X_MAX: i32 = WORLD_WIDTH - 10;
    const CONFIG: VacationConfig = VacationConfig::standard(&[
        (StatId::Serenity, 8.0),
        (StatId::Fulfillment, 8.0),
    ]);
    // Indoor scene, no sky. Tanks sit against a black wall.
    const HAS_SKY: bool = false;

    fn enter(&mut self, _ctx: &mut GameContext, _base: &mut LocationScene) {
        self.rng = (Instant::now().duration_since_epoch().as_micros() as u32).max(1);
        self.spawn_creatures();
    }

    fn exit(&mut self, _ctx: &mut GameContext) {
        self.fish.clear();
        self.octopus = None;
        self.bubbles = None;
        self.debris = None;
    }

    fn tick(&mut self, _ctx: &mut GameContext, scaled_dt: f32) {
        self.sw_timer += scaled_dt;
        if self.sw_timer >= SW_FRAME_INTERVAL {
            self.sw_timer -= SW_FRAME_INTERVAL;
            self.sw_frame = (self.sw_frame + 1) % SW_FRAMES;
        }
        for f in self.fish.iter_mut() {
            f.update(scaled_dt);
        }
        if let Some(oct) = self.octopus.as_mut() {
            oct.update(scaled_dt);
        }
        if let Some(b) = self.bubbles.as_mut() {
            b.update(scaled_dt);
        }
        if let Some(d) = self.debris.as_mut() {
            d.update(scaled_dt);
        }
    }

    fn draw_world(&self, _ctx: &GameContext, renderer: &mut Renderer, base: &LocationScene) {
        base.environment.draw_layer(renderer, Layer::Background);
        base.environment.draw_layer(renderer, Layer::Midground);
        // Midground custom passes: rocks -> seaweed -> creatures -> particles ->
        // occluders (paint over anything that escaped the windows) -> outlines.
        self.draw_rocks_and_plants(renderer, base);
        self.draw_seaweed(renderer, base);
        self.draw_creatures(renderer, base);
        self.draw_particles(renderer, base);
        self.draw_occluders(renderer, base);
        self.draw_tank_outlines(renderer, base);
        base.environment.draw_layer(renderer, Layer::Foreground);
    }
}
