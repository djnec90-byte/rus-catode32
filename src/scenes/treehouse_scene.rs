use embedded_graphics::prelude::{Point, Size};

use crate::{
    assets::{furniture::CAT_BED_SIDE, nature::COBWEB},
    context::GameContext,
    environment::Layer,
    input::Buttons,
    location_scene::LocationScene,
    render::{Renderer, SpriteOpts},
    scene::{Scene, SceneId},
};

const WORLD_WIDTH: i32 = 256;
const CHAR_WORLD_X: i32 = 64;
const CHAR_WORLD_Y: i32 = 64;
const CAT_BED_WORLD_X: i32 = 130;
const COBWEB_WORLD_X: i32 = 156;

pub struct TreehouseScene {
    base: LocationScene,
}

impl TreehouseScene {
    pub fn new() -> Self {
        Self {
            base: LocationScene::new(WORLD_WIDTH, Point::new(CHAR_WORLD_X, CHAR_WORLD_Y)),
        }
    }

    fn draw_platform_fg(&self, renderer: &mut Renderer) {
        let fg_offset = self.base.environment.camera_offset(Layer::Foreground);
        let sx = (0 - fg_offset).max(0);
        let ex = (WORLD_WIDTH - fg_offset).min(128);
        if sx >= ex {
            return;
        }
        for py in [59, 61] {
            renderer.draw_line(Point::new(sx, py), Point::new(ex, py));
        }
        for &world_x in &[10, 80, 160, 245] {
            let px = world_x - fg_offset;
            if sx <= px && px <= ex {
                renderer.draw_line(Point::new(px, 59), Point::new(px, 63));
            }
        }
    }

    fn draw_platform_mid(&self, renderer: &mut Renderer) {
        let mg_offset = self.base.environment.camera_offset(Layer::Midground);
        let sx = (0 - mg_offset).max(0);
        let ex = (300 - mg_offset).min(128);
        if sx >= ex {
            return;
        }
        renderer.draw_line(Point::new(sx, 57), Point::new(ex, 57));
        for &world_x in &[20, 90, 160, 230] {
            let px = world_x - mg_offset;
            if sx <= px && px <= ex {
                renderer.draw_line(Point::new(px, 57), Point::new(px, 59));
            }
        }
        // Tree trunks framing the platform
        renderer.fill_rect_off(Point::new(0 - mg_offset - 1, -1), Size::new(10, 61));
        renderer.fill_rect_off(Point::new(180 - mg_offset, -1), Size::new(10, 61));
        renderer.draw_rect(Point::new(0 - mg_offset - 1, -1), Size::new(10, 61), false);
        renderer.draw_rect(Point::new(180 - mg_offset, -1), Size::new(10, 61), false);
        // Cobweb tucked in the top-right corner of the platform.
        renderer.draw_sprite(
            &COBWEB,
            Point::new(COBWEB_WORLD_X - mg_offset, 0),
            SpriteOpts::default(),
        );
    }

    fn draw_cat_bed_rim(&self, renderer: &mut Renderer) {
        let fg_offset = self.base.environment.camera_offset(Layer::Foreground);
        let bed_x = CAT_BED_WORLD_X - fg_offset;
        let w = CAT_BED_SIDE.width as i32;
        renderer.draw_sprite(&CAT_BED_SIDE, Point::new(bed_x, 52), SpriteOpts::default());
        renderer.fill_rect_off(Point::new(bed_x + w, 54), Size::new(20, 10));
        renderer.draw_line(Point::new(bed_x + w, 54), Point::new(bed_x + w + 20, 54));
        renderer.draw_line(Point::new(bed_x + w, 63), Point::new(bed_x + w + 20, 63));
        renderer.draw_line(Point::new(bed_x + w, 59), Point::new(bed_x + w + 20, 59));
        renderer.draw_sprite(
            &CAT_BED_SIDE,
            Point::new(bed_x + w + 20, 52),
            SpriteOpts {
                mirror_h: true,
                ..Default::default()
            },
        );
    }
}

impl Scene for TreehouseScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.base.enter(ctx, SceneId::Treehouse);
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
        // TODO: character.set_pose("sitting.forward.neutral") on enter (Python override).
        // TODO: context.cat_bed_x tracking for sleep-in-bed pose adjustments.
        // TODO: espnow.start() when WiFi is wired up and not currently visiting.
        // TODO: weather-change detection (Python re-enters scene on weather change).
        // TODO: plant surfaces (PLANT_SURFACES) once the plant system is ported.
        None
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        if self.base.menu_active() {
            self.base.draw_menu(renderer);
            return;
        }
        // Open-air — sky visible.
        self.base.draw_sky(renderer, ctx);
        self.base.environment.draw_layer(renderer, Layer::Background);
        self.base.environment.draw_layer(renderer, Layer::Midground);
        self.draw_platform_mid(renderer);
        self.base.environment.draw_layer(renderer, Layer::Foreground);
        self.draw_platform_fg(renderer);
        self.base.draw_character(renderer, ctx);
        self.draw_cat_bed_rim(renderer);
    }
}
