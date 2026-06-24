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
    temperature_system::get_temperature,
    time_system::Weather,
    weather_system::{ForecastEntry, WeatherSystem, FORECAST_MAX_ENTRIES},
};

const COL_W: i32 = 26;
const LABEL_Y: i32 = 13;
const ICON_Y: i32 = 25;
const ICON_X_OFF: i32 = (COL_W - 15) / 2;
const METEOR_Y: i32 = 43;
const METEOR_X_OFF: i32 = (COL_W - 13) / 2;
const NUM_SLOTS: usize = 24;
const INTERVAL_H: u32 = 3;
const VIS_COLS: usize = (128 / COL_W) as usize;
const HL_Y: i32 = 12;
const HL_H: u32 = (ICON_Y + 17 - HL_Y) as u32;
const CHAR_W: i32 = 6; // FONT_6X10 width

#[derive(Clone, Copy)]
struct Slot {
    hour: u8,
    weather: Weather,
    shower: bool,
    temp_c: f32,
}

pub struct ForecastScene {
    slots: heapless::Vec<Slot, NUM_SLOTS>,
    cursor: usize,
    scroll: usize,
}

impl ForecastScene {
    pub fn new() -> Self {
        Self {
            slots: heapless::Vec::new(),
            cursor: 0,
            scroll: 0,
        }
    }
}

fn fmt_hour(h: u8, out: &mut String<4>) {
    out.clear();
    if h == 0 {
        let _ = out.push_str(t!("12A"));
    } else if h < 12 {
        let _ = write!(out, "{}A", h);
    } else if h == 12 {
        let _ = out.push_str(t!("12P"));
    } else {
        let _ = write!(out, "{}P", h - 12);
    }
}

fn fmt_temp(temp: f32, out: &mut String<6>) {
    out.clear();
    let rounded = if temp >= 0.0 {
        (temp + 0.5) as i32
    } else {
        -((-temp + 0.5) as i32)
    };
    let _ = write!(out, "{}C", rounded);
}

fn day_icon(weather: Weather) -> &'static [u8] {
    match weather {
        Weather::Clear => icons::WEATHER_CLEAR,
        Weather::Cloudy => icons::WEATHER_CLOUDY,
        Weather::Overcast => icons::WEATHER_OVERCAST,
        Weather::Rain => icons::WEATHER_RAIN,
        Weather::Storm => icons::WEATHER_STORM,
        Weather::Snow => icons::WEATHER_SNOW,
        Weather::Windy => icons::WEATHER_WINDY,
    }
}

fn night_icon(weather: Weather) -> Option<&'static [u8]> {
    match weather {
        Weather::Clear => Some(icons::WEATHER_CLEAR_NIGHT),
        Weather::Cloudy => Some(icons::WEATHER_CLOUDY_NIGHT),
        Weather::Overcast => Some(icons::WEATHER_OVERCAST_NIGHT),
        _ => None,
    }
}

fn icon_for(weather: Weather, hour: u8) -> &'static [u8] {
    let is_night = hour >= 20 || hour < 6;
    if is_night {
        if let Some(n) = night_icon(weather) {
            return n;
        }
    }
    day_icon(weather)
}

fn build_slots(
    forecast: &[ForecastEntry],
    cur_hour: u8,
    cur_min: u8,
    cur_day: u32,
    season_offset: u16,
    pet_seed: u32,
) -> heapless::Vec<Slot, NUM_SLOTS> {
    let cur_total: i32 = cur_hour as i32 * 60 + cur_min as i32;
    let interval_min: i32 = INTERVAL_H as i32 * 60;
    let slot0_start: i32 = (cur_total / interval_min) * interval_min;
    let mut slots: heapless::Vec<Slot, NUM_SLOTS> = heapless::Vec::new();

    for i in 0..NUM_SLOTS {
        let slot_start = slot0_start + i as i32 * interval_min - cur_total;
        let slot_end = slot_start + interval_min;
        let eff_start = slot_start.max(0);
        let slot_abs = slot0_start + i as i32 * interval_min;
        let slot_hour = ((slot_abs / 60).rem_euclid(24)) as u8;
        let slot_day = cur_day as i64 + (slot_abs as i64).div_euclid(24 * 60);

        let mut cumul: i32 = 0;
        let mut weather = forecast.first().map(|e| e.weather).unwrap_or(Weather::Clear);
        let mut shower = false;

        for entry in forecast {
            let next_cumul = cumul + entry.duration_minutes as i32;
            if next_cumul > eff_start && cumul < slot_end {
                if cumul <= eff_start {
                    weather = entry.weather;
                }
                if entry.meteor_shower {
                    shower = true;
                }
            }
            cumul = next_cumul;
            if cumul >= slot_end {
                break;
            }
        }

        let temp = get_temperature(
            slot_day.max(0) as u32,
            season_offset,
            slot_hour,
            weather,
            pet_seed,
        );
        let _ = slots.push(Slot {
            hour: slot_hour,
            weather,
            shower,
            temp_c: temp,
        });
    }
    slots
}

impl Scene for ForecastScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        let ws = WeatherSystem::new();
        let forecast: heapless::Vec<ForecastEntry, FORECAST_MAX_ENTRIES> =
            ws.get_forecast(ctx, 72);
        // Use the lower 32 bits of the adopted pet's seed for forecast jitter.
        let pet_seed: u32 = ctx.pet_seed as u32;
        self.slots = build_slots(
            &forecast,
            ctx.time_hours,
            ctx.time_minutes,
            ctx.day_number,
            ctx.season_offset,
            pet_seed,
        );
        self.cursor = 0;
        self.scroll = 0;
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::Left) {
            if self.cursor > 0 {
                self.cursor -= 1;
                if self.cursor < self.scroll {
                    self.scroll = self.cursor;
                }
            }
        } else if buttons.was_just_pressed(Button::Right) {
            if self.cursor + 1 < self.slots.len() {
                self.cursor += 1;
                if self.cursor >= self.scroll + VIS_COLS {
                    self.scroll = self.cursor + 1 - VIS_COLS;
                }
            }
        } else if buttons.was_just_pressed(Button::A) || buttons.was_just_pressed(Button::B) {
            return Some(ctx.last_main_scene);
        }
        None
    }

    fn draw(&self, ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        if self.slots.is_empty() {
            return;
        }

        // Header: "Season: Weather" for the highlighted slot.
        let sel = self.slots[self.cursor];
        let mut header: String<32> = String::new();
        let _ = write!(header, "{}: {}", ctx.season.name(), sel.weather.name());
        renderer.draw_text(header.as_str(), Point::new(0, 0));
        renderer.draw_line(Point::new(0, 9), Point::new(127, 9));

        // Hourly columns, one extra to fill any partial column on the right.
        let end = (self.scroll + VIS_COLS + 1).min(self.slots.len());
        for (i, slot) in self.slots[self.scroll..end].iter().enumerate() {
            let col_x = i as i32 * COL_W;

            let mut label: String<4> = String::new();
            fmt_hour(slot.hour, &mut label);
            let label_x = col_x + (COL_W - label.len() as i32 * CHAR_W) / 2;
            renderer.draw_text(label.as_str(), Point::new(label_x, LABEL_Y));

            renderer.draw_sprite_raw(
                icon_for(slot.weather, slot.hour),
                15,
                15,
                Point::new(col_x + ICON_X_OFF, ICON_Y),
                SpriteOpts::default(),
            );

            let is_night = slot.hour >= 20 || slot.hour < 6;
            let sky_clear = matches!(
                slot.weather,
                Weather::Clear | Weather::Cloudy | Weather::Windy
            );
            if slot.shower && is_night && sky_clear {
                renderer.draw_sprite_raw(
                    icons::METEOR_SHOWER,
                    13,
                    13,
                    Point::new(col_x + METEOR_X_OFF, METEOR_Y),
                    SpriteOpts::default(),
                );
            }
        }

        // Highlight rectangle around the cursor column.
        let hl_x = (self.cursor - self.scroll) as i32 * COL_W;
        renderer.draw_rect(Point::new(hl_x, HL_Y), Size::new(COL_W as u32, HL_H), false);

        // Scroll indicators.
        if self.scroll > 0 {
            renderer.draw_text("<", Point::new(0, 55));
        }
        if self.scroll + VIS_COLS < self.slots.len() {
            renderer.draw_text(">", Point::new(120, 55));
        }

        // Centred temperature for the selected slot.
        let mut temp_str: String<6> = String::new();
        fmt_temp(sel.temp_c, &mut temp_str);
        let temp_x = (128 - temp_str.len() as i32 * CHAR_W) / 2;
        renderer.draw_text(temp_str.as_str(), Point::new(temp_x, 55));
    }
}
