use crate::{context::GameContext, weather_system::WeatherSystem};

// Matches Python's production override in main.py: game_minutes_per_second = 1/15.
// That gives ~15 real minutes per in-game hour and ~6 real hours per in-game day.
const GAME_MINUTES_PER_SECOND: f32 = 1.0 / 15.0;

pub const MOON_PHASES: [&str; 8] = [
    "New", "Wax Cres", "1st Qtr", "Wax Gib", "Full", "Wan Gib", "3rd Qtr", "Wan Cres",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Season {
    Winter,
    Spring,
    Summer,
    Fall,
}

impl Season {
    pub fn name(self) -> &'static str {
        match self {
            Season::Winter => "Winter",
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall => "Fall",
        }
    }

    pub fn for_day(day: u32, offset: u16) -> Self {
        let d = (day + offset as u32) % 365;
        if d < 60 {
            Season::Winter
        } else if d < 152 {
            Season::Spring
        } else if d < 265 {
            Season::Summer
        } else if d < 356 {
            Season::Fall
        } else {
            Season::Winter
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weather {
    Clear,
    Cloudy,
    Overcast,
    Windy,
    Rain,
    Storm,
    Snow,
}

impl Weather {
    pub fn name(self) -> &'static str {
        match self {
            Weather::Clear => "Clear",
            Weather::Cloudy => "Cloudy",
            Weather::Overcast => "Overcast",
            Weather::Windy => "Windy",
            Weather::Rain => "Rain",
            Weather::Storm => "Storm",
            Weather::Snow => "Snow",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Weather::Clear => Weather::Cloudy,
            Weather::Cloudy => Weather::Overcast,
            Weather::Overcast => Weather::Windy,
            Weather::Windy => Weather::Rain,
            Weather::Rain => Weather::Storm,
            Weather::Storm => Weather::Snow,
            Weather::Snow => Weather::Clear,
        }
    }
}

pub struct TimeSystem {
    accumulator: f32,
    last_temp_hour: i8,
    // TODO: set from ctx.pet_seed once the personality system is ported.
    pub pet_seed: u64,
    weather_system: WeatherSystem,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            accumulator: 0.0,
            last_temp_hour: -1,
            pet_seed: 0,
            weather_system: WeatherSystem::new(),
        }
    }

    pub fn advance(&mut self, ctx: &mut GameContext, dt: f32) {
        let dt = dt * ctx.time_speed;
        self.accumulator += dt * GAME_MINUTES_PER_SECOND;
        if self.accumulator < 1.0 {
            return;
        }
        let mins_to_add = self.accumulator as u32;
        self.accumulator -= mins_to_add as f32;

        let total_minutes = ctx.time_minutes as u32 + mins_to_add;
        let old_hours = ctx.time_hours as u32;
        let new_hours_raw = old_hours + total_minutes / 60;
        ctx.time_hours = (new_hours_raw % 24) as u8;
        ctx.time_minutes = (total_minutes % 60) as u8;

        if new_hours_raw >= 24 {
            ctx.day_number += new_hours_raw / 24;
            ctx.moon_phase = ((ctx.day_number / 6 + 2) % 8) as u8;
            ctx.season = Season::for_day(ctx.day_number, ctx.season_offset);
        }

        let current_hour = ctx.time_hours as i8;
        if current_hour != self.last_temp_hour {
            self.last_temp_hour = current_hour;
            // TODO: port temperature_system.get_temperature using
            // pet_seed + day + offset + hour + weather.
        }

        self.weather_system.update(mins_to_add, ctx);
    }
}
