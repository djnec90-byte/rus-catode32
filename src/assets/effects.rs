//! Shared particle / effect sprites. Port of the `BURST1` (and friends)
//! statics in `micropython/src/assets/effects.py`.

use crate::render::Sprite;

// BURST1 — 5-frame outline sparkle that radiates outward then dissipates.
// Played at 8 fps (frame_dur = 0.125s, total per particle = 0.625s) to match
// the Python `speed=8` field.
const BURST1_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x10\x38\x10\x00\x00",
    b"\x00\x10\x10\x7c\x10\x10\x00",
    b"\x10\x10\x28\xc6\x28\x10\x10",
    b"\x10\x44\x00\x82\x00\x44\x10",
    b"\x82\x00\x00\x00\x00\x00\x82",
];

pub static BURST1: Sprite = Sprite {
    width: 7,
    height: 7,
    frames: BURST1_FRAMES,
    fill_frames: None,
};

pub const BURST1_FRAME_DUR: f32 = 1.0 / 8.0;
