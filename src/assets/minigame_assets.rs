use crate::render::Sprite;

const PAW_SMALL1_FRAMES: &[&[u8]] = &[
    b"\x28\xaa\x82\x38\x7c\x7c\x28",
];

pub static PAW_SMALL1: Sprite = Sprite {
    width: 7,
    height: 7,
    frames: PAW_SMALL1_FRAMES,
    fill_frames: None,
};

// Snake minigame head frames. The two frames have different dimensions,
// so they don't fit a single `Sprite` (fixed width/height) and are drawn
// via `draw_sprite_raw` with explicit dimensions per direction.
pub const CAT_THIN_DOWN: &[u8] = b"\x84\xcc\xfc\xb4\xb4\xfc\x78\x30"; // 6x8
pub const CAT_THIN_RIGHT: &[u8] = b"\xfc\x66\x3f\x3f\x66\xfc"; // 8x6

const SPOT_FRAMES: &[&[u8]] = &[b"\x60\xf0\xf0\x60"];

pub static SPOT: Sprite = Sprite {
    width: 4,
    height: 4,
    frames: SPOT_FRAMES,
    fill_frames: None,
};
