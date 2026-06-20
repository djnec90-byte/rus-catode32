use crate::{
    context::GameContext,
    input::Buttons,
    render::Renderer,
    scene::{Scene, SceneId},
    time_system::{Season, Weather, MOON_PHASES},
    ui::settings::{SettingItem, SettingValue, Settings, SettingsResult},
};

const SEASONS: &[&str] = &["Spring", "Summer", "Fall", "Winter"];
const WEATHERS: &[&str] = &[
    "Clear", "Cloudy", "Overcast", "Windy", "Rain", "Storm", "Snow",
];

const DAY_IDX: usize = 0;
const HOUR_IDX: usize = 1;
const MIN_IDX: usize = 2;
const SEASON_IDX: usize = 3;
const MOON_IDX: usize = 4;
const WEATHER_IDX: usize = 5;
const TEMP_IDX: usize = 6;

fn season_to_index(s: Season) -> usize {
    match s {
        Season::Spring => 0,
        Season::Summer => 1,
        Season::Fall => 2,
        Season::Winter => 3,
    }
}

fn season_from_index(i: usize) -> Season {
    match i {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Fall,
        _ => Season::Winter,
    }
}

fn weather_to_index(w: Weather) -> usize {
    match w {
        Weather::Clear => 0,
        Weather::Cloudy => 1,
        Weather::Overcast => 2,
        Weather::Windy => 3,
        Weather::Rain => 4,
        Weather::Storm => 5,
        Weather::Snow => 6,
    }
}

fn weather_from_index(i: usize) -> Weather {
    match i {
        0 => Weather::Clear,
        1 => Weather::Cloudy,
        2 => Weather::Overcast,
        3 => Weather::Windy,
        4 => Weather::Rain,
        5 => Weather::Storm,
        _ => Weather::Snow,
    }
}

pub struct DebugEnvScene {
    settings: Settings,
}

impl DebugEnvScene {
    pub fn new() -> Self {
        Self {
            settings: Settings::new(),
        }
    }

    fn open_with(&mut self, ctx: &GameContext) {
        let items = [
            SettingItem::int("Day", ctx.day_number as i32, 0, 9_999_999, 1),
            SettingItem::int("Hour", ctx.time_hours as i32, 0, 23, 1),
            SettingItem::int("Min", ctx.time_minutes as i32, 0, 55, 5),
            SettingItem::choice("Season", season_to_index(ctx.season), SEASONS),
            SettingItem::choice("Moon", ctx.moon_phase as usize, &MOON_PHASES),
            SettingItem::choice("Weather", weather_to_index(ctx.weather), WEATHERS),
            SettingItem::int("Temp", ctx.temperature as i32, -20, 50, 1),
        ];
        self.settings.open(&items);
    }

    fn apply_values(&self, ctx: &mut GameContext) {
        if let Some(SettingValue::Int { value, .. }) = self.settings.value(DAY_IDX) {
            ctx.day_number = (*value).max(0) as u32;
        }
        if let Some(SettingValue::Int { value, .. }) = self.settings.value(HOUR_IDX) {
            ctx.time_hours = (*value).clamp(0, 23) as u8;
        }
        if let Some(SettingValue::Int { value, .. }) = self.settings.value(MIN_IDX) {
            ctx.time_minutes = (*value).clamp(0, 59) as u8;
        }
        if let Some(SettingValue::Choice { index, .. }) = self.settings.value(SEASON_IDX) {
            ctx.season = season_from_index(*index);
        }
        if let Some(SettingValue::Choice { index, .. }) = self.settings.value(MOON_IDX) {
            ctx.moon_phase = (*index).min(MOON_PHASES.len() - 1) as u8;
        }
        if let Some(SettingValue::Choice { index, .. }) = self.settings.value(WEATHER_IDX) {
            ctx.weather = weather_from_index(*index);
        }
        if let Some(SettingValue::Int { value, .. }) = self.settings.value(TEMP_IDX) {
            ctx.temperature = *value as f32;
        }
        ctx.weather_timer = 60.0;
    }
}

impl Scene for DebugEnvScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.open_with(ctx);
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        _dt: f32,
    ) -> Option<SceneId> {
        match self.settings.handle_input(buttons) {
            SettingsResult::Continue | SettingsResult::Activated(_) => None,
            SettingsResult::Closed => {
                self.apply_values(ctx);
                Some(ctx.last_main_scene)
            }
        }
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        self.settings.draw(renderer);
    }
}
