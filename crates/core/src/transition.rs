//! Screen transition effects between scenes and around sleep entry/exit.
//!
//! Two-phase scanline-interlace fade: an `Out` phase closes the screen to 
//! black in 8 passes, then a one-frame `Midpoint` is signalled so the
//! caller can perform the deferred work (scene swap, sleep entry), then
//! an `In` phase opens it back up.

use embedded_graphics::prelude::{Point, Size};

use crate::board::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::render::Renderer;

/// Cap dt so a slow scene load doesn't cause the reveal to skip frames.
const FPS: f32 = 12.0;
const MAX_DT: f32 = 1.0 / FPS;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Out,
    In,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransitionStep {
    Inactive,
    Active,
    /// Fired exactly once after the `Out` phase finishes with one fully-black
    /// frame on screen. The caller should perform any deferred work (scene
    /// swap, sleep entry) before returning
    Midpoint,
}

pub struct TransitionManager {
    duration: f32,
    active: bool,
    progress: f32,
    phase: Phase,
    /// Set when the `Out` phase reaches full coverage. Holds the fully-black
    /// frame visible for one tick before the midpoint fires, so the player
    /// sees a clean black frame and not the near-complete dither.
    ready_for_midpoint: bool,
    midpoint_fired: bool,
}

impl TransitionManager {
    pub const fn new() -> Self {
        Self {
            duration: 0.2,
            active: false,
            progress: 0.0,
            phase: Phase::Out,
            ready_for_midpoint: false,
            midpoint_fired: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn start(&mut self) -> bool {
        if self.active {
            return false;
        }
        self.active = true;
        self.phase = Phase::Out;
        self.progress = 0.0;
        self.ready_for_midpoint = false;
        self.midpoint_fired = false;
        true
    }

    /// Begin the opening phase only. No `Out`, no midpoint signal. Used
    /// after waking from sleep where the screen was already black.
    pub fn start_in_only(&mut self) {
        self.active = true;
        self.phase = Phase::In;
        self.progress = 0.0;
        self.ready_for_midpoint = false;
        self.midpoint_fired = true;
    }

    pub fn update(&mut self, dt: f32) -> TransitionStep {
        if !self.active {
            return TransitionStep::Inactive;
        }

        if self.ready_for_midpoint {
            self.ready_for_midpoint = false;
            self.phase = Phase::In;
            self.progress = 0.0;
            if !self.midpoint_fired {
                self.midpoint_fired = true;
                return TransitionStep::Midpoint;
            }
            return TransitionStep::Active;
        }

        let dt = if dt > MAX_DT { MAX_DT } else { dt };
        self.progress += dt / self.duration;

        if self.progress >= 1.0 {
            self.progress = 1.0;
            match self.phase {
                Phase::Out => self.ready_for_midpoint = true,
                Phase::In => {
                    self.active = false;
                    self.progress = 0.0;
                }
            }
        }
        TransitionStep::Active
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        if !self.active {
            return;
        }

        let progress = match self.phase {
            Phase::Out => self.progress,
            Phase::In => 1.0 - self.progress,
        };

        if progress <= 0.0 {
            return;
        }
        if progress >= 1.0 {
            renderer.fill_rect_off(
                Point::new(0, 0),
                Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32),
            );
            return;
        }

        // 8-pass scanline interlace: each pass clears every 8th row at a
        // different offset, adding ~12.5 % coverage per pass.
        let passes = (progress * 8.0) as i32 + 1;
        if passes >= 8 {
            renderer.fill_rect_off(
                Point::new(0, 0),
                Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32),
            );
        } else {
            for offset in 0..passes {
                let mut y = offset;
                while y < DISPLAY_HEIGHT as i32 {
                    renderer.fill_rect_off(
                        Point::new(0, y),
                        Size::new(DISPLAY_WIDTH as u32, 1),
                    );
                    y += 8;
                }
            }
        }
    }
}
