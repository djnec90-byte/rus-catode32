use esp_hal::time::{Duration, Instant};

use crate::{
    context::GameContext,
    input::Buttons,
    render::Renderer,
    scene::{SceneId, SceneManager},
    time_system::TimeSystem,
};

const FPS: u64 = 12;
const FRAME_TIME_MS: u64 = 1000 / FPS;

pub struct Game {
    renderer: Renderer,
    buttons: Buttons,
    context: GameContext,
    scene_manager: SceneManager,
    time_system: TimeSystem,
    last_dt_ms: u64,
}

impl Game {
    pub fn new(renderer: Renderer, buttons: Buttons) -> Self {
        let mut context = GameContext::new();
        let scene_manager = SceneManager::new(&mut context, SceneId::Inside);
        Self {
            renderer,
            buttons,
            context,
            scene_manager,
            time_system: TimeSystem::new(),
            last_dt_ms: 0,
        }
    }

    pub fn run(&mut self) -> ! {
        let mut last_frame = Instant::now();
        loop {
            let frame_start = Instant::now();
            let elapsed_ms = (frame_start.duration_since_epoch()
                - last_frame.duration_since_epoch())
            .as_millis();
            last_frame = frame_start;
            self.last_dt_ms = elapsed_ms;

            let dt = elapsed_ms as f32 / 1000.0;
            self.update(dt);
            self.draw();

            let frame_used = frame_start.elapsed().as_millis();
            if frame_used < FRAME_TIME_MS {
                let wait_start = Instant::now();
                let remaining = Duration::from_millis(FRAME_TIME_MS - frame_used);
                while wait_start.elapsed() < remaining {}
            }
        }
    }

    fn update(&mut self, dt: f32) {
        self.time_system.advance(&mut self.context, dt);
        self.context.tick(dt);
        self.scene_manager
            .update(&mut self.context, &mut self.buttons, dt);
    }

    fn draw(&mut self) {
        self.renderer.clear();
        // Baseline; scenes that want lightning override this within their draw.
        self.renderer.set_invert(false);
        self.scene_manager
            .draw(&self.context, &mut self.renderer, self.last_dt_ms);
        self.renderer.flush();
    }
}
