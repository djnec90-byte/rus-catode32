use embedded_graphics::prelude::Point;
use crate::platform::time::Instant;
use heapless::Vec;

use crate::{
    assets::{
        furniture::{PARK_BENCH, STREET_LAMP},
        nature::BUSH,
        plants::{GRASS_GROWING, GRASS_YOUNG, TINY_FLOWER},
    },
    context::{GameContext, StatId},
    entities::flyer::{FlyerEntity, FlyerKind},
    environment::Layer,
    location_scene::LocationScene,
    rand::{rand_range_f32, rand_range_u32},
    render::{Renderer, Sprite, SpriteOpts},
    scene::SceneId,
    scenes::vacation_base::{VacationConfig, VacationScene, VacationWorld},
};

const WORLD_WIDTH: i32 = 300;
const GROUND_Y: i32 = 63;
const CHAR_WORLD_X: i32 = 64;

const PATH_Y_TOP: i32 = 44;
const PATH_Y_BOT: i32 = 48;

const LAMP_POSITIONS: &[i32] = &[20, 90, 160];
const BENCH_POSITIONS: &[i32] = &[55, 125];

// (world_x, y_bottom), bottoms 2-3px above PATH_Y_TOP, drawn before benches/lamps.
const BUSH_POSITIONS: &[(i32, i32)] = &[(8, 42), (42, 41), (88, 42), (120, 41)];

const PATH_GRASS_X: &[i32] = &[5, 12, 28, 33, 51, 67, 72, 89, 98, 119, 138, 142, 161];

// (world_x, sprite, y_bottom)
struct Scatter {
    x: i32,
    sprite: &'static Sprite,
    y_bottom: i32,
}

const SCATTER_FOREGROUND: &[Scatter] = &[
    Scatter { x: 15, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 55, sprite: &GRASS_GROWING, y_bottom: 63 },
    Scatter { x: 105, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 140, sprite: &GRASS_GROWING, y_bottom: 63 },
    Scatter { x: 185, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 220, sprite: &GRASS_GROWING, y_bottom: 63 },
    Scatter { x: 255, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 285, sprite: &GRASS_GROWING, y_bottom: 63 },
];

const SCATTER_MIDGROUND: &[Scatter] = &[
    Scatter { x: 42, sprite: &TINY_FLOWER, y_bottom: 58 },
    Scatter { x: 90, sprite: &TINY_FLOWER, y_bottom: 55 },
    Scatter { x: 125, sprite: &GRASS_YOUNG, y_bottom: 60 },
    Scatter { x: 165, sprite: &TINY_FLOWER, y_bottom: 57 },
    Scatter { x: 175, sprite: &TINY_FLOWER, y_bottom: 53 },
    Scatter { x: 230, sprite: &TINY_FLOWER, y_bottom: 59 },
    Scatter { x: 240, sprite: &GRASS_YOUNG, y_bottom: 56 },
    Scatter { x: 275, sprite: &TINY_FLOWER, y_bottom: 54 },
];

const MAX_BUTTERFLIES: usize = 4;

#[derive(Default)]
pub struct ParkWorld {
    butterflies: Vec<FlyerEntity, MAX_BUTTERFLIES>,
    rng: u32,
}

pub type VacationParkScene = VacationScene<ParkWorld>;

impl ParkWorld {
    fn spawn_butterflies(&mut self) {
        self.butterflies.clear();
        let count = rand_range_u32(&mut self.rng, 2, 3);
        for _ in 0..count {
            if self.butterflies.is_full() {
                break;
            }
            let x = rand_range_f32(&mut self.rng, 20.0, (WORLD_WIDTH - 20) as f32);
            let y = rand_range_f32(&mut self.rng, 5.0, 35.0);
            let _ = self
                .butterflies
                .push(FlyerEntity::new(FlyerKind::Butterfly, x, y, WORLD_WIDTH, &mut self.rng));
        }
    }

    fn place_background_objects(base: &mut LocationScene) {
        for &(x, y_bot) in BUSH_POSITIONS {
            base.environment.add_object(
                Layer::Background,
                &BUSH,
                x,
                y_bot - BUSH.height as i32,
                false,
            );
        }
        for &x in BENCH_POSITIONS {
            base.environment.add_object(
                Layer::Background,
                &PARK_BENCH,
                x,
                PATH_Y_TOP - PARK_BENCH.height as i32,
                false,
            );
        }
        for &x in LAMP_POSITIONS {
            base.environment.add_object(
                Layer::Background,
                &STREET_LAMP,
                x,
                PATH_Y_TOP - STREET_LAMP.height as i32,
                false,
            );
        }
    }

    fn draw_path(renderer: &mut Renderer, base: &LocationScene) {
        let bg_offset = base.environment.camera_offset(Layer::Background);
        for &wx in PATH_GRASS_X {
            let sx = wx - bg_offset;
            if (0..128).contains(&sx) {
                renderer.draw_pixel(Point::new(sx, PATH_Y_TOP - 1), true);
            }
        }
        // Two parallel feathered lines: a slow modulus picks gap zones and a
        // fast modulus adds occasional feather/skip pixels. Independent
        // constants per line keep them visually decorrelated.
        for sx in 0..128 {
            let wx = sx + bg_offset;
            let slow_t = (wx * 3).rem_euclid(53);
            let fast_t = (wx * 11).rem_euclid(7);
            if slow_t < 11 {
                if fast_t == 0 {
                    renderer.draw_pixel(Point::new(sx, PATH_Y_TOP), true);
                }
            } else if fast_t != 2 {
                renderer.draw_pixel(Point::new(sx, PATH_Y_TOP), true);
            }
            let slow_b = (wx * 5).rem_euclid(61);
            let fast_b = (wx * 9).rem_euclid(7);
            if slow_b < 13 {
                if fast_b == 0 {
                    renderer.draw_pixel(Point::new(sx, PATH_Y_BOT), true);
                }
            } else if fast_b != 3 {
                renderer.draw_pixel(Point::new(sx, PATH_Y_BOT), true);
            }
        }
    }

    fn draw_scatter(renderer: &mut Renderer, base: &LocationScene, layer: Layer, items: &[Scatter]) {
        let offset = base.environment.camera_offset(layer);
        for s in items {
            let sx = s.x - offset;
            let w = s.sprite.width as i32;
            if sx + w < 0 || sx >= 128 {
                continue;
            }
            renderer.draw_sprite(
                s.sprite,
                Point::new(sx, s.y_bottom - s.sprite.height as i32),
                SpriteOpts::default(),
            );
        }
    }

    fn draw_butterflies(&self, renderer: &mut Renderer, base: &LocationScene) {
        let offset = base.environment.camera_offset(Layer::Foreground);
        for b in &self.butterflies {
            b.draw(renderer, offset);
        }
    }
}

impl VacationWorld for ParkWorld {
    const SCENE_ID: SceneId = SceneId::VacationPark;
    const WORLD_WIDTH: i32 = WORLD_WIDTH;
    const CHAR_WORLD_X: i32 = CHAR_WORLD_X;
    const GROUND_Y: i32 = GROUND_Y;
    const X_MIN: i32 = 10;
    const X_MAX: i32 = WORLD_WIDTH - 10;
    const CONFIG: VacationConfig = VacationConfig::standard(&[
        (StatId::Fulfillment, 8.0),
        (StatId::Playfulness, 8.0),
    ]);
    const HAS_SKY: bool = true;

    fn enter(&mut self, _ctx: &mut GameContext, base: &mut LocationScene) {
        self.rng = (Instant::now().duration_since_epoch().as_micros() as u32).max(1);
        Self::place_background_objects(base);
        self.spawn_butterflies();
    }

    fn tick(&mut self, _ctx: &mut GameContext, dt: f32) {
        for b in self.butterflies.iter_mut() {
            b.update(dt);
        }
    }

    fn draw_world(&self, _ctx: &GameContext, renderer: &mut Renderer, base: &LocationScene) {
        Self::draw_path(renderer, base);
        base.environment.draw_layer(renderer, Layer::Background);
        base.environment.draw_layer(renderer, Layer::Midground);
        Self::draw_scatter(renderer, base, Layer::Midground, SCATTER_MIDGROUND);
        base.environment.draw_layer(renderer, Layer::Foreground);
        Self::draw_scatter(renderer, base, Layer::Foreground, SCATTER_FOREGROUND);
        self.draw_butterflies(renderer, base);
    }
}
