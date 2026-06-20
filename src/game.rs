use esp_hal::{
    peripherals::{GPIO0, GPIO1, GPIO2, GPIO3, LPWR},
    rng::Rng,
    rtc_cntl::{
        sleep::{Ext1WakeupSource, WakeupLevel},
        Rtc,
    },
    system::software_reset,
    time::{Duration, Instant},
};
use esp_println::println;

use crate::{
    context::{GameContext, PowerAction},
    input::{Button, Buttons},
    led::Led,
    render::Renderer,
    scene::{SceneId, SceneManager},
    sleep_manager::{wait_buttons_stable_released_mask, SleepManager},
    time_system::TimeSystem,
};

const DEEP_WAKE_BUTTONS: [Button; 4] = [Button::A, Button::B, Button::Menu1, Button::Menu2];

/// Stubbed save-file check. Always returns `false` until the save/load layer
/// is ported — the boot path therefore always lands in the adoption flow.
/// TODO(save_load): consult the persisted save file once that layer exists.
fn has_save() -> bool {
    false
}

const FPS: u64 = 12;
const FRAME_TIME_MS: u64 = 1000 / FPS;

pub struct Game {
    renderer: Renderer,
    buttons: Buttons,
    context: GameContext,
    scene_manager: SceneManager,
    time_system: TimeSystem,
    sleep_manager: SleepManager,
    last_dt_ms: u64,
}

impl Game {
    pub fn new(renderer: Renderer, buttons: Buttons, rng: Rng, led: Led) -> Self {
        let mut context = GameContext::new(led);
        // Seed the behavior RNG from the hardware peripheral so each boot's
        // behavior choices differ until/unless a save provides a seed.
        let seed = rng.random();
        context.rng = if seed == 0 { 1 } else { seed };
        context.hw_rng = rng;
        let start = if has_save() {
            SceneId::Inside
        } else {
            SceneId::Adoption
        };
        let scene_manager = SceneManager::new(&mut context, start);
        Self {
            renderer,
            buttons,
            context,
            scene_manager,
            time_system: TimeSystem::new(),
            sleep_manager: SleepManager::new(),
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

            // Reset the inactivity timer whenever the player is touching a button.
            if self.buttons.any_pressed() {
                self.sleep_manager.notify_activity();
            }

            self.update(dt);
            self.draw();

            if let Some(action) = self.context.pending_power.take() {
                self.handle_power_action(action);
            }

            let frame_used = frame_start.elapsed().as_millis();
            if frame_used < FRAME_TIME_MS {
                let wait_start = Instant::now();
                let remaining = Duration::from_millis(FRAME_TIME_MS - frame_used);
                while wait_start.elapsed() < remaining {}
            }

            // Enter basic sleep if idle long enough. Blocks here until a
            // button press wakes the device; on return, reset the frame
            // timer so the next iteration doesn't see a huge dt spike
            // (the sleep loop already advanced time_system internally).
            if self.sleep_manager.should_sleep() {
                self.sleep_manager.enter_sleep(
                    &mut self.renderer,
                    &mut self.buttons,
                    &mut self.context,
                    &mut self.scene_manager,
                    &mut self.time_system,
                );
                last_frame = Instant::now();
            }
        }
    }

    fn update(&mut self, dt: f32) {
        self.time_system.advance(&mut self.context, dt);
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

    fn handle_power_action(&mut self, action: PowerAction) {
        match action {
            PowerAction::Reboot => {
                println!("[Power] Software reset");
                software_reset();
            }
            PowerAction::LightSleep => {
                println!("[Power] Light sleep (basic mode)");
                self.sleep_manager.enter_sleep(
                    &mut self.renderer,
                    &mut self.buttons,
                    &mut self.context,
                    &mut self.scene_manager,
                    &mut self.time_system,
                );
            }
            PowerAction::DeepSleep => self.enter_deep_sleep(),
        }
    }

    /// One-way deep sleep. Wakes the device only on a falling edge on
    /// GPIO0/1/2/3 (A/B/Menu1/Menu2 buttons) — direction buttons live on
    /// GPIO14/18/19/20, outside the C6's LP-IO domain, so they cannot
    /// serve as wake sources. The device resets on wake.
    fn enter_deep_sleep(&mut self) -> ! {
        println!("[Power] Entering deep sleep");
        self.renderer.clear();
        self.renderer.flush();
        self.renderer.power_off();

        // Wait for the wake-capable buttons to be released and stable —
        // otherwise Ext1's level=Low trigger fires immediately on either
        // the still-held press or its release bounce.
        wait_buttons_stable_released_mask(
            &self.buttons,
            &DEEP_WAKE_BUTTONS,
            Duration::from_millis(500),
        );

        // Safety: deep sleep is one-way; on wake the device boots from reset
        // and the old `Input` wrappers in `Buttons` will never be used again.
        let mut g0 = unsafe { GPIO0::steal() };
        let mut g1 = unsafe { GPIO1::steal() };
        let mut g2 = unsafe { GPIO2::steal() };
        let mut g3 = unsafe { GPIO3::steal() };

        // The IO_MUX pull-ups configured at boot don't survive Ext1's
        // `pad_hold` switch to RTC mode — without an RTC-side pull-up the
        // LP-IO pin floats and the level=Low trigger fires almost
        // immediately. Enable them on the LP-IO peripheral now, before
        // Ext1::apply runs and freezes the pad state.
        use esp_hal::gpio::RtcPinWithResistors;
        g0.rtcio_pullup(true);
        g1.rtcio_pullup(true);
        g2.rtcio_pullup(true);
        g3.rtcio_pullup(true);

        let mut pins: [(&mut dyn RtcPinWithResistors, WakeupLevel); 4] = [
            (&mut g0, WakeupLevel::Low),
            (&mut g1, WakeupLevel::Low),
            (&mut g2, WakeupLevel::Low),
            (&mut g3, WakeupLevel::Low),
        ];
        let ext1 = Ext1WakeupSource::new(&mut pins);

        let mut rtc = Rtc::new(unsafe { LPWR::steal() });
        rtc.sleep_deep(&[&ext1]);
    }
}
