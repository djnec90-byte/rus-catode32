pub struct GameContext {
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
}

impl GameContext {
    pub fn new() -> Self {
        Self {
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
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt * self.time_speed;
        self.fullness = (self.fullness - 3.3 * dt).max(0.0);
        self.energy = (self.energy - 2.0 * dt).max(0.0);
        self.comfort = (self.comfort - 1.0 * dt).max(0.0);
        self.playfulness = (self.playfulness - 1.5 * dt).max(0.0);
        self.focus = (self.focus - 2.5 * dt).max(0.0);
    }
}
