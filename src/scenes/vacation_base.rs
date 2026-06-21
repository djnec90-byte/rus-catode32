//! Shared vacation timer + stat-accrual / overstay-penalty logic.
//!
//! Used by the four vacation scenes (park, forest, aquarium, beach) to keep
//! the per-scene file focused on art. Each scene owns a `VacationState` and
//! calls `tick()` from its update path; `apply_rewards_on_exit()` lands the
//! accumulated bonuses, and `cleanup_context()` clears the cross-scene flags.

use crate::context::{GameContext, StatId};

#[derive(Clone, Copy)]
pub struct VacationConfig {
    /// Seconds until the enjoyment cap is reached and the pet starts asking
    /// to go home.
    pub enjoy_duration: f32,
    /// Seconds of grace after the cap before overstay penalties begin.
    pub grace_duration: f32,
    /// Stat totals awarded on exit, scaled to time spent up to the cap.
    pub accrual: &'static [(StatId, f32)],
    /// Per-second penalties applied during overstay (negative deltas).
    pub penalties: &'static [(StatId, f32)],
}

const PENALTY_INTERVAL: f32 = 90.0;

pub struct VacationState {
    pub config: VacationConfig,
    timer: f32,
    home_wanted: bool,
    penalty_accum: f32,
}

impl VacationState {
    pub fn new(config: VacationConfig) -> Self {
        Self {
            config,
            timer: 0.0,
            home_wanted: false,
            penalty_accum: 0.0,
        }
    }

    /// Mirrors Python `VacationScene.enter`: stamps the cross-scene flags so
    /// the behavior layer (auto-pick scene-exit, vocalizing) sees them.
    pub fn on_enter(&mut self, ctx: &mut GameContext) {
        self.timer = 0.0;
        self.home_wanted = false;
        self.penalty_accum = 0.0;
        ctx.on_vacation = true;
        ctx.wants_to_go_home = false;
    }

    /// Mirrors Python `VacationScene.exit`: lands the proportional reward
    /// (only if any time was spent) and clears the vacation flags so the
    /// next scene's behavior cycle is unaffected.
    pub fn on_exit(&mut self, ctx: &mut GameContext) {
        self.apply_rewards(ctx);
        ctx.on_vacation = false;
        ctx.wants_to_go_home = false;
    }

    pub fn tick(&mut self, ctx: &mut GameContext, dt: f32) {
        let prev = self.timer;
        self.timer += dt;

        // Crossed the enjoyment cap this frame → start the "go home" nag.
        if prev < self.config.enjoy_duration && self.timer >= self.config.enjoy_duration {
            self.home_wanted = true;
            ctx.wants_to_go_home = true;
        }

        // Overstay penalties — accumulate dt and apply in batches so the
        // penalty fires noticeably rather than being lost to per-frame
        // damping inside apply_stat_changes.
        if !self.config.penalties.is_empty()
            && self.timer >= self.config.enjoy_duration + self.config.grace_duration
        {
            self.penalty_accum += dt;
            if self.penalty_accum >= PENALTY_INTERVAL {
                let scale = self.penalty_accum;
                self.penalty_accum = 0.0;
                let mut batched: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
                for &(stat, per_sec) in self.config.penalties {
                    let _ = batched.push((stat, per_sec * scale));
                }
                ctx.apply_stat_changes(&batched);
            }
        }
    }

    fn apply_rewards(&self, ctx: &mut GameContext) {
        if self.config.accrual.is_empty() {
            return;
        }
        let proportion = (self.timer / self.config.enjoy_duration).clamp(0.0, 1.0);
        if proportion <= 0.0 {
            return;
        }
        let mut batched: heapless::Vec<(StatId, f32), 8> = heapless::Vec::new();
        for &(stat, total) in self.config.accrual {
            let _ = batched.push((stat, total * proportion));
        }
        ctx.apply_stat_changes(&batched);
    }
}
