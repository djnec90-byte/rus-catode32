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

const PAW_MED_FRAMES: &[&[u8]] = &[
    b"\x0a\x00\x1b\x00\x5b\x40\xd1\x60\xc4\x60\x9f\x20\x3f\x80\x3f\x80\x7f\xc0\x7f\xc0\x7f\xc0\x3f\x80\x1b\x00",
];

pub static PAW_MED: Sprite = Sprite {
    width: 11,
    height: 13,
    frames: PAW_MED_FRAMES,
    fill_frames: None,
};

const PAW_LARGE1_FRAMES: &[&[u8]] = &[
    b"\x04\x40\x0c\x60\x0e\xe0\x1e\xf0\x5c\x74\x5c\x74\xe8\x2e\xe0\x0e\xe7\xce\x4f\xe4\x1f\xf0\x1f\xf0\x3f\xf8\x3f\xf8\x3f\xf8\x1f\xf0\x0c\x60",
];

pub static PAW_LARGE1: Sprite = Sprite {
    width: 15,
    height: 17,
    frames: PAW_LARGE1_FRAMES,
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
