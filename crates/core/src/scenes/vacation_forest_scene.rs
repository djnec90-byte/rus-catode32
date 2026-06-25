use embedded_graphics::prelude::{Point, Size};
use crate::platform::time::Instant;
use heapless::Vec;

use crate::{
    assets::{
        nature::{BUSH, TREE_TRUNK, TREE_TRUNK_SMALL},
        plants::{GRASS_GROWING, GRASS_YOUNG},
    },
    context::{GameContext, StatId},
    entities::{
        flyer::{FlyerEntity, FlyerKind},
        jumper::{JumperEntity, JumperKind},
    },
    environment::Layer,
    gardening_ui::PlantSurface,
    input::Buttons,
    location_scene::LocationScene,
    rand::{rand_bool, rand_range_f32, rand_range_u32},
    render::{Renderer, Sprite, SpriteOpts},
    scene::{Scene, SceneId},
    scenes::vacation_base::{VacationConfig, VacationState},
};

const PLANT_SURFACES: &[PlantSurface] = &[];

const WORLD_WIDTH: i32 = 300;
const GROUND_Y: i32 = 63;
const CHAR_WORLD_X: i32 = 64;

// Trunk column geometry: outline pixels per trunk style.
const TREE_LEFT_LINE: i32 = 6;
const TREE_RIGHT_LINE: i32 = 16;
const SMALL_TREE_LEFT_LINE: i32 = 4;
const SMALL_TREE_RIGHT_LINE: i32 = 8;

// Top of the trunk column (interior fill extends from y=0 down to this).
const BG_TRUNK_TOP_Y: i32 = 32;
const BG_SMALL_TRUNK_TOP_Y: i32 = 36;
const MG_TRUNK_TOP_Y: i32 = 42;

/// Bark detail line. (x_offset_from_trunk_left, y0, y1).
struct BarkLine(i32, i32, i32);

struct TreePlacement {
    x: i32,
    bark: &'static [BarkLine],
}

const BG_LARGE_TREE_BARK_0: &[BarkLine] = &[BarkLine(8, 20, 24), BarkLine(10, 28, 29)];
const BG_LARGE_TREE_BARK_1: &[BarkLine] = &[BarkLine(8, 8, 16), BarkLine(10, 25, 26)];
const BG_LARGE_TREE_BARK_2: &[BarkLine] = &[BarkLine(8, 22, 25), BarkLine(10, 10, 21)];

const BG_LARGE_TREES: &[TreePlacement] = &[
    TreePlacement { x: 18, bark: BG_LARGE_TREE_BARK_0 },
    TreePlacement { x: 82, bark: BG_LARGE_TREE_BARK_1 },
    TreePlacement { x: 150, bark: BG_LARGE_TREE_BARK_2 },
];

const BG_SMALL_TREE_BARK_0: &[BarkLine] = &[BarkLine(6, 23, 27), BarkLine(6, 33, 33), BarkLine(6, 13, 10)];
const BG_SMALL_TREE_BARK_1: &[BarkLine] = &[BarkLine(6, 26, 30), BarkLine(6, 21, 21), BarkLine(6, 35, 35)];
const BG_SMALL_TREE_BARK_2: &[BarkLine] = &[BarkLine(6, 24, 28), BarkLine(6, 35, 35), BarkLine(6, 12, 8)];
const BG_SMALL_TREE_BARK_3: &[BarkLine] = &[BarkLine(6, 28, 32), BarkLine(6, 10, 8), BarkLine(6, 34, 34)];

const BG_SMALL_TREES: &[TreePlacement] = &[
    TreePlacement { x: 50, bark: BG_SMALL_TREE_BARK_0 },
    TreePlacement { x: 108, bark: BG_SMALL_TREE_BARK_1 },
    TreePlacement { x: 132, bark: BG_SMALL_TREE_BARK_2 },
    TreePlacement { x: 165, bark: BG_SMALL_TREE_BARK_3 },
];

const MG_TREE_BARK_0: &[BarkLine] = &[BarkLine(8, 30, 35), BarkLine(10, 38, 41)];
const MG_TREE_BARK_1: &[BarkLine] = &[BarkLine(8, 27, 31), BarkLine(10, 36, 38)];

const MG_TREES: &[TreePlacement] = &[
    TreePlacement { x: 45, bark: MG_TREE_BARK_0 },
    TreePlacement { x: 175, bark: MG_TREE_BARK_1 },
];

const BG_BUSH_POSITIONS: &[(i32, i32)] = &[
    (5, 45), (35, 43), (70, 46), (100, 44), (130, 45), (160, 43),
];
const MG_BUSH_POSITIONS: &[(i32, i32)] = &[
    (20, 55), (75, 54), (130, 55), (185, 54), (235, 55),
];

struct Scatter {
    x: i32,
    sprite: &'static Sprite,
    y_bottom: i32,
}

const SCATTER_FOREGROUND: &[Scatter] = &[
    Scatter { x: 10, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 45, sprite: &GRASS_GROWING, y_bottom: 63 },
    Scatter { x: 85, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 120, sprite: &GRASS_GROWING, y_bottom: 63 },
    Scatter { x: 160, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 195, sprite: &GRASS_GROWING, y_bottom: 63 },
    Scatter { x: 235, sprite: &GRASS_YOUNG, y_bottom: 63 },
    Scatter { x: 270, sprite: &GRASS_GROWING, y_bottom: 63 },
    Scatter { x: 295, sprite: &GRASS_YOUNG, y_bottom: 63 },
];

const SCATTER_MIDGROUND: &[Scatter] = &[
    Scatter { x: 30, sprite: &GRASS_YOUNG, y_bottom: 56 },
    Scatter { x: 80, sprite: &GRASS_GROWING, y_bottom: 55 },
    Scatter { x: 115, sprite: &GRASS_YOUNG, y_bottom: 57 },
    Scatter { x: 165, sprite: &GRASS_GROWING, y_bottom: 54 },
    Scatter { x: 210, sprite: &GRASS_YOUNG, y_bottom: 56 },
    Scatter { x: 260, sprite: &GRASS_GROWING, y_bottom: 55 },
];

const MAX_CRITTERS: usize = 12;

enum Critter {
    Flyer(FlyerEntity),
    Jumper(JumperEntity),
}

const CONFIG: VacationConfig = VacationConfig {
    enjoy_duration: 750.0,
    grace_duration: 120.0,
    accrual: &[
        (StatId::Serenity, 8.0),
        (StatId::Fulfillment, 8.0),
    ],
    penalties: &[
        (StatId::Comfort, -0.005),
        (StatId::Serenity, -0.003),
    ],
};

pub struct VacationForestScene {
    base: LocationScene,
    state: VacationState,
    critters: Vec<Critter, MAX_CRITTERS>,
    rng: u32,
}

impl VacationForestScene {
    pub fn new() -> Self {
        Self {
            base: LocationScene::new(WORLD_WIDTH, Point::new(CHAR_WORLD_X, GROUND_Y)),
            state: VacationState::new(CONFIG),
            critters: Vec::new(),
            rng: 1,
        }
    }

    fn place_bushes(&mut self) {
        for &(x, y_bot) in BG_BUSH_POSITIONS {
            self.base.environment.add_object(
                Layer::Background,
                &BUSH,
                x,
                y_bot - BUSH.height as i32,
                false,
            );
        }
        for &(x, y_bot) in MG_BUSH_POSITIONS {
            self.base.environment.add_object(
                Layer::Midground,
                &BUSH,
                x,
                y_bot - BUSH.height as i32,
                false,
            );
        }
    }

    fn spawn_critters(&mut self, ctx: &GameContext) {
        self.critters.clear();
        let day = ctx.is_daytime();
        if day {
            let n = rand_range_u32(&mut self.rng, 1, 3);
            for _ in 0..n {
                self.add_flyer(FlyerKind::Butterfly, 5.0, 40.0);
            }
        } else {
            let n = rand_range_u32(&mut self.rng, 1, 3);
            for _ in 0..n {
                self.add_flyer(FlyerKind::Moth, 5.0, 40.0);
            }
            let n = rand_range_u32(&mut self.rng, 1, 3);
            for _ in 0..n {
                self.add_flyer(FlyerKind::Firefly, 30.0, 55.0);
            }
        }
        let n = rand_range_u32(&mut self.rng, 1, 2);
        for _ in 0..n {
            self.add_jumper(JumperKind::Grasshopper);
        }
    }

    fn add_flyer(&mut self, kind: FlyerKind, y_min: f32, y_max: f32) {
        if self.critters.is_full() {
            return;
        }
        let x = rand_range_f32(&mut self.rng, 20.0, (WORLD_WIDTH - 20) as f32);
        let y = rand_range_f32(&mut self.rng, y_min, y_max);
        let _ = self.critters.push(Critter::Flyer(FlyerEntity::new(
            kind,
            x,
            y,
            WORLD_WIDTH,
            &mut self.rng,
        )));
    }

    fn add_jumper(&mut self, kind: JumperKind) {
        if self.critters.is_full() {
            return;
        }
        let direction: i8 = if rand_bool(&mut self.rng, 0.5) { 1 } else { -1 };
        let x = if direction == 1 {
            -12.0
        } else {
            (WORLD_WIDTH + 12) as f32
        };
        let _ = self
            .critters
            .push(Critter::Jumper(JumperEntity::new(kind, x, direction, &mut self.rng)));
    }

    fn update_critters(&mut self, dt: f32) {
        for c in self.critters.iter_mut() {
            match c {
                Critter::Flyer(f) => f.update(dt),
                Critter::Jumper(j) => j.update(dt),
            }
        }
        self.critters.retain(|c| match c {
            Critter::Jumper(j) => !j.despawned,
            _ => true,
        });
    }

    fn draw_trees(
        &self,
        renderer: &mut Renderer,
        layer: Layer,
        placements: &[TreePlacement],
        trunk_top_y: i32,
        sprite: &'static Sprite,
        left_line: i32,
        right_line: i32,
    ) {
        let offset = self.base.environment.camera_offset(layer);
        let fill_x = left_line + 1;
        let fill_w = (right_line - left_line - 1).max(0) as u32;
        let w = sprite.width as i32;
        for p in placements {
            let sx = p.x - offset;
            if sx + w < 0 || sx >= 128 {
                continue;
            }
            // Black-fill column from screen top down to the trunk top, then
            // the two outline lines bracketing it, then bark detail, then the
            // trunk sprite anchored at trunk_top_y.
            renderer.fill_rect_off(
                Point::new(sx + fill_x, 0),
                Size::new(fill_w, trunk_top_y.max(0) as u32),
            );
            renderer.draw_line(
                Point::new(sx + left_line, 0),
                Point::new(sx + left_line, trunk_top_y - 1),
            );
            renderer.draw_line(
                Point::new(sx + right_line, 0),
                Point::new(sx + right_line, trunk_top_y - 1),
            );
            for line in p.bark {
                renderer.draw_line(
                    Point::new(sx + line.0, line.1),
                    Point::new(sx + line.0, line.2),
                );
            }
            renderer.draw_sprite(sprite, Point::new(sx, trunk_top_y), SpriteOpts::default());
        }
    }

    fn draw_scatter(&self, renderer: &mut Renderer, layer: Layer, items: &[Scatter]) {
        let offset = self.base.environment.camera_offset(layer);
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

    fn draw_critters(&self, renderer: &mut Renderer) {
        let offset = self.base.environment.camera_offset(Layer::Foreground);
        for c in &self.critters {
            match c {
                Critter::Flyer(f) => f.draw(renderer, offset),
                Critter::Jumper(j) => j.draw(renderer, offset),
            }
        }
    }
}

impl Scene for VacationForestScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.base.enter(ctx, SceneId::VacationForest, PLANT_SURFACES);
        ctx.scene_x_min = 10;
        ctx.scene_x_max = WORLD_WIDTH - 10;
        self.rng = (Instant::now().duration_since_epoch().as_micros() as u32).max(1);
        self.place_bushes();
        self.spawn_critters(ctx);
        self.state.on_enter(ctx);
    }

    fn exit(&mut self, ctx: &mut GameContext) {
        self.state.on_exit(ctx);
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
        let scaled = dt * ctx.time_speed;
        self.update_critters(scaled);
        self.state.tick(ctx, scaled);
        None
    }

    fn tick_background(&mut self, ctx: &mut GameContext, dt: f32) {
        self.base.tick_background(ctx, dt);
        let scaled = dt * ctx.time_speed;
        self.update_critters(scaled);
        self.state.tick(ctx, scaled);
    }

    fn mark_behavior_almost_done(&mut self, ctx: &mut GameContext) {
        self.base.mark_behavior_almost_done(ctx);
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        if self.base.menu_active() {
            self.base.draw_menu(renderer);
            return;
        }
        self.base.draw_sky(renderer, ctx);
        self.base.environment.draw_layer(renderer, Layer::Background);
        self.draw_trees(
            renderer,
            Layer::Background,
            BG_LARGE_TREES,
            BG_TRUNK_TOP_Y,
            &TREE_TRUNK,
            TREE_LEFT_LINE,
            TREE_RIGHT_LINE,
        );
        self.draw_trees(
            renderer,
            Layer::Background,
            BG_SMALL_TREES,
            BG_SMALL_TRUNK_TOP_Y,
            &TREE_TRUNK_SMALL,
            SMALL_TREE_LEFT_LINE,
            SMALL_TREE_RIGHT_LINE,
        );
        self.base.environment.draw_layer(renderer, Layer::Midground);
        self.draw_trees(
            renderer,
            Layer::Midground,
            MG_TREES,
            MG_TRUNK_TOP_Y,
            &TREE_TRUNK,
            TREE_LEFT_LINE,
            TREE_RIGHT_LINE,
        );
        self.draw_scatter(renderer, Layer::Midground, SCATTER_MIDGROUND);
        self.base.environment.draw_layer(renderer, Layer::Foreground);
        self.draw_scatter(renderer, Layer::Foreground, SCATTER_FOREGROUND);
        self.draw_critters(renderer);
        self.base.draw_character(renderer, ctx);
        self.base.draw_overlay(ctx, renderer);
    }
}
