use core::fmt::Write;

use embedded_graphics::prelude::{Point, Size};
use heapless::String;
use crate::t;

use crate::{
    assets::icons,
    context::GameContext,
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
    scene::{Scene, SceneId},
    ui::popup::Popup,
};

const COLS: usize = 6;
const ROWS: usize = 3;
const CELL_W: i32 = 17;
const CELL_H: i32 = 20;
const H_GAP: i32 = 4;
const V_GAP: i32 = 2;
const CELL_STEP_X: i32 = CELL_W + H_GAP;
const CELL_STEP_Y: i32 = CELL_H + V_GAP;
const GRID_X: i32 = 3;
const GRID_Y: i32 = 0;

#[derive(Clone, Copy)]
enum StatKey {
    Health,
    Fullness,
    Energy,
    Comfort,
    Cleanliness,
    Fitness,
    Focus,
    Intelligence,
    Curiosity,
    Playfulness,
    Affection,
    Fulfillment,
    Serenity,
    Sociability,
    Courage,
    Loyalty,
    Mischievousness,
    Maturity,
}

impl StatKey {
    fn value(self, ctx: &GameContext) -> f32 {
        match self {
            StatKey::Health => ctx.health,
            StatKey::Fullness => ctx.fullness,
            StatKey::Energy => ctx.energy,
            StatKey::Comfort => ctx.comfort,
            StatKey::Cleanliness => ctx.cleanliness,
            StatKey::Fitness => ctx.fitness,
            StatKey::Focus => ctx.focus,
            StatKey::Intelligence => ctx.intelligence,
            StatKey::Curiosity => ctx.curiosity,
            StatKey::Playfulness => ctx.playfulness,
            StatKey::Affection => ctx.affection,
            StatKey::Fulfillment => ctx.fulfillment,
            StatKey::Serenity => ctx.serenity,
            StatKey::Sociability => ctx.sociability,
            StatKey::Courage => ctx.courage,
            StatKey::Loyalty => ctx.loyalty,
            StatKey::Mischievousness => ctx.mischievousness,
            StatKey::Maturity => ctx.maturity,
        }
    }
}

struct StatDef {
    name: &'static str,
    icon: &'static [u8],
    desc: &'static str,
    key: StatKey,
}

const STATS: [StatDef; 18] = [
    StatDef { name: t!("Health"), icon: icons::HEALTH, key: StatKey::Health,
        desc: t!("desc_health") },
    StatDef { name: t!("Fullness"), icon: icons::FULLNESS, key: StatKey::Fullness,
        desc: t!("desc_fullness") },
    StatDef { name: t!("Energy"), icon: icons::ENERGY, key: StatKey::Energy,
        desc: t!("desc_energy") },
    StatDef { name: t!("Comfort"), icon: icons::COMFORT, key: StatKey::Comfort,
        desc: t!("desc_comfort") },
    StatDef { name: t!("Cleanliness"), icon: icons::CLEANLINESS, key: StatKey::Cleanliness,
        desc: t!("desc_cleanliness") },
    StatDef { name: t!("Fitness"), icon: icons::FITNESS, key: StatKey::Fitness,
        desc: t!("desc_fitness") },

    StatDef { name: t!("Focus"), icon: icons::FOCUS, key: StatKey::Focus,
        desc: t!("desc_focus") },
    StatDef { name: t!("Intelligence"), icon: icons::INTELLIGENCE, key: StatKey::Intelligence,
        desc: t!("desc_intelligence") },
    StatDef { name: t!("Curiosity"), icon: icons::CURIOSITY, key: StatKey::Curiosity,
        desc: t!("desc_curiosity") },
    StatDef { name: t!("Playfulness"), icon: icons::PLAYFULNESS, key: StatKey::Playfulness,
        desc: t!("desc_playfulness") },
    StatDef { name: t!("Affection"), icon: icons::AFFECTION, key: StatKey::Affection,
        desc: t!("desc_affection") },
    StatDef { name: t!("Fulfillment"), icon: icons::FULFILLMENT, key: StatKey::Fulfillment,
        desc: t!("desc_fulfillment") },

    StatDef { name: t!("Serenity"), icon: icons::SERENITY, key: StatKey::Serenity,
        desc: t!("desc_serenity") },
    StatDef { name: t!("Sociability"), icon: icons::SOCIABILITY, key: StatKey::Sociability,
        desc: t!("desc_sociability") },
    StatDef { name: t!("Courage"), icon: icons::COURAGE, key: StatKey::Courage,
        desc: t!("desc_courage") },
    StatDef { name: t!("Loyalty"), icon: icons::LOYALTY, key: StatKey::Loyalty,
        desc: t!("desc_loyalty") },
    StatDef { name: t!("Mischievousness"), icon: icons::MISCHIEVOUS, key: StatKey::Mischievousness,
        desc: t!("desc_mischievousness") },
    StatDef { name: t!("Maturity"), icon: icons::MATURITY, key: StatKey::Maturity,
        desc: t!("desc_maturity") },
];

pub struct StatsScene {
    sel_col: usize,
    sel_row: usize,
    showing_detail: bool,
    popup: Popup,
}

impl StatsScene {
    pub fn new() -> Self {
        Self {
            sel_col: 0,
            sel_row: 0,
            showing_detail: false,
            popup: Popup::new(0, 6, 128, 48),
        }
    }

    fn selected_index(&self) -> usize {
        self.sel_row * COLS + self.sel_col
    }
}

impl Scene for StatsScene {
    fn enter(&mut self, _ctx: &mut GameContext) {
        self.sel_col = 0;
        self.sel_row = 0;
        self.showing_detail = false;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        if self.showing_detail {
            if buttons.was_just_pressed(Button::Up) {
                self.popup.scroll_up();
            }
            if buttons.was_just_pressed(Button::Down) {
                self.popup.scroll_down();
            }
            if buttons.was_just_pressed(Button::B) || buttons.was_just_pressed(Button::A) {
                self.showing_detail = false;
            }
            return None;
        }

        if buttons.was_just_pressed(Button::Left) {
            if self.sel_col > 0 {
                self.sel_col -= 1;
            } else if self.sel_row > 0 {
                self.sel_row -= 1;
                self.sel_col = COLS - 1;
            }
        }
        if buttons.was_just_pressed(Button::Right) {
            if self.sel_col < COLS - 1 {
                self.sel_col += 1;
            } else if self.sel_row < ROWS - 1 {
                self.sel_row += 1;
                self.sel_col = 0;
            }
        }
        if buttons.was_just_pressed(Button::Up) && self.sel_row > 0 {
            self.sel_row -= 1;
        }
        if buttons.was_just_pressed(Button::Down) && self.sel_row < ROWS - 1 {
            self.sel_row += 1;
        }

        if buttons.was_just_pressed(Button::A) {
            let stat = &STATS[self.selected_index()];
            let value = stat.key.value(ctx) as i32;
            let mut buf: String<256> = String::new();
            let _ = write!(buf, "{}\n---- {}% ----\n{}", stat.name, value, stat.desc);
            self.popup.set_text(buf.as_str(), true, false);
            self.showing_detail = true;
            return None;
        }

        if buttons.was_just_pressed(Button::B) {
            return Some(ctx.last_main_scene);
        }

        None
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        for row in 0..ROWS {
            for col in 0..COLS {
                let idx = row * COLS + col;
                let stat = &STATS[idx];
                let bx = GRID_X + col as i32 * CELL_STEP_X;
                let by = GRID_Y + row as i32 * CELL_STEP_Y;
                let is_selected = col == self.sel_col && row == self.sel_row;
                let value = stat.key.value(ctx);
                draw_cell(renderer, bx, by, stat.icon, value, is_selected);
            }
        }

        if self.showing_detail {
            self.popup.draw(renderer, true);
        }
    }
}

fn draw_cell(renderer: &mut Renderer, bx: i32, by: i32, icon: &[u8], value: f32, is_selected: bool) {
    renderer.draw_rect(
        Point::new(bx, by),
        Size::new(CELL_W as u32, CELL_H as u32),
        false,
    );
    renderer.draw_line(Point::new(bx, by + 16), Point::new(bx + CELL_W - 1, by + 16));

    if is_selected {
        renderer.draw_rect(Point::new(bx + 2, by + 2), Size::new(13, 13), true);
        renderer.draw_sprite_raw(
            icon,
            13,
            13,
            Point::new(bx + 2, by + 2),
            SpriteOpts {
                transparent: true,
                invert: true,
                transparent_color: true,
                ..Default::default()
            },
        );
    } else {
        renderer.draw_sprite_raw(
            icon,
            13,
            13,
            Point::new(bx + 2, by + 2),
            SpriteOpts::default(),
        );
    }

    let bar_w = (value.clamp(0.0, 100.0) / 100.0 * (CELL_W - 2) as f32) as u32;
    if bar_w > 0 {
        renderer.draw_rect(Point::new(bx + 1, by + 17), Size::new(bar_w, 2), true);
    }
}
