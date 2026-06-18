use crate::time_system::{Season, Weather};

pub struct GameContext {
    pub health: f32,
    pub fullness: f32,
    pub energy: f32,
    pub comfort: f32,
    pub playfulness: f32,
    pub focus: f32,

    pub fulfillment: f32,
    pub cleanliness: f32,
    pub intelligence: f32,
    pub maturity: f32,
    pub affection: f32,

    pub fitness: f32,
    pub serenity: f32,

    pub courage: f32,
    pub loyalty: f32,
    pub mischievousness: f32,
    pub curiosity: f32,
    pub sociability: f32,

    pub sickness: f32,

    pub time_speed: f32,

    pub coins: i32,

    pub zoomies_high_score: i32,
    pub maze_best_time: i32,
    pub snake_high_score: i32,
    pub memory_best_score: i32,
    pub hanjie_best_time: i32,

    // World/environment state — advanced by TimeSystem each frame.
    pub time_hours: u8,
    pub time_minutes: u8,
    pub day_number: u32,
    pub season_offset: u16,
    pub season: Season,
    pub moon_phase: u8,
    pub weather: Weather,
    pub temperature: f32,
    pub weather_step: u32,
    pub weather_timer: f32,        // in-game minutes remaining in current weather
    pub meteor_shower_timer: f32,  // in-game minutes of active shower window
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            health: 50.0,
            fullness: 50.0,
            energy: 50.0,
            comfort: 50.0,
            playfulness: 50.0,
            focus: 50.0,
            fulfillment: 50.0,
            cleanliness: 50.0,
            intelligence: 50.0,
            maturity: 50.0,
            affection: 50.0,
            fitness: 50.0,
            serenity: 50.0,
            courage: 50.0,
            loyalty: 50.0,
            mischievousness: 50.0,
            curiosity: 50.0,
            sociability: 50.0,
            sickness: 0.0,
            time_speed: 1.0,
            coins: 50,
            zoomies_high_score: 0,
            maze_best_time: 0,
            snake_high_score: 0,
            memory_best_score: -1,
            hanjie_best_time: -1,

            time_hours: 0,
            time_minutes: 0,
            day_number: 0,
            // TODO: derive season_offset from ctx.pet_seed once the personality system is ported.
            season_offset: 0,
            season: Season::Winter,
            moon_phase: 2, // (0/6 + 2) % 8 — initial value matches day 0
            weather: Weather::Clear,
            temperature: 20.0,
            weather_step: 0,
            weather_timer: 0.0,
            meteor_shower_timer: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt * self.time_speed;
        self.fullness = (self.fullness - 3.3 * dt).max(0.0);
        self.energy = (self.energy - 2.0 * dt).max(0.0);
        self.comfort = (self.comfort - 1.0 * dt).max(0.0);
        self.playfulness = (self.playfulness - 1.5 * dt).max(0.0);
        self.focus = (self.focus - 2.5 * dt).max(0.0);
        self.recompute_health();
    }

    pub fn recompute_health(&mut self) {
        let raw = 0.25 * self.fullness
            + 0.20 * self.fitness
            + 0.20 * self.energy
            + 0.15 * self.cleanliness
            + 0.05 * self.comfort
            + 0.05 * self.affection
            + 0.025 * self.fulfillment
            + 0.025 * self.focus
            + 0.025 * self.intelligence
            + 0.025 * self.playfulness;
        self.health = raw.clamp(0.0, 100.0);
    }
}
