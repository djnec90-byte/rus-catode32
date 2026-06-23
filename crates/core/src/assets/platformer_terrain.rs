//! Terrain tile sprites for the Prowl platformer minigame.
//! All terrain tiles are 8x8.

use crate::render::Sprite;

// Tile type constants: used as indices into TERRAIN_TILES. Must match the
// values in the level converter.
pub const TILE_TOP: u8 = 0;
pub const TILE_TOP_LEFT: u8 = 1;
pub const TILE_TOP_RIGHT: u8 = 2;
pub const TILE_SIDE_LEFT: u8 = 3;
pub const TILE_SIDE_RIGHT: u8 = 4;
pub const TILE_BOTTOM: u8 = 5;
pub const TILE_BOTTOM_LEFT: u8 = 6;
pub const TILE_BOTTOM_RIGHT: u8 = 7;
pub const TILE_TOP_BOTTOM: u8 = 8;
pub const TILE_TOP_LEFT_BOTTOM: u8 = 9;
pub const TILE_TOP_RIGHT_BOTTOM: u8 = 10;
pub const TILE_LEFT_RIGHT: u8 = 11;
pub const TILE_LEFT_RIGHT_BOTTOM: u8 = 12;
pub const TILE_TOP_LEFT_RIGHT: u8 = 13;
pub const TILE_TOP_LEFT_BOTTOM_RIGHT: u8 = 14;

const TERRAIN_TOP_FRAMES: &[&[u8]] = &[b"\xff\x00\x77\x77\x00\xa1\x00\x00"];
pub static TERRAIN_TOP: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_FRAMES, fill_frames: None,
};
const TERRAIN_TOP2_FRAMES: &[&[u8]] = &[b"\xff\x00\x36\x40\x00\x04\x00\x00"];
pub static TERRAIN_TOP2: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP2_FRAMES, fill_frames: None,
};

const TERRAIN_TOP_LEFT_FRAMES: &[&[u8]] = &[b"\x7f\x80\xb7\xb7\x80\x88\x40\x20"];
pub static TERRAIN_TOP_LEFT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_LEFT_FRAMES, fill_frames: None,
};

const TERRAIN_TOP_RIGHT_FRAMES: &[&[u8]] = &[b"\xfe\x01\x75\x75\x01\xa1\x02\x04"];
pub static TERRAIN_TOP_RIGHT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_RIGHT_FRAMES, fill_frames: None,
};

const TERRAIN_SIDE_LEFT_FRAMES: &[&[u8]] = &[b"\x40\x50\x40\x40\x40\x40\x48\x40"];
pub static TERRAIN_SIDE_LEFT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_SIDE_LEFT_FRAMES, fill_frames: None,
};

const TERRAIN_SIDE_RIGHT_FRAMES: &[&[u8]] = &[b"\x02\x0a\x42\x02\x02\x02\x12\x02"];
pub static TERRAIN_SIDE_RIGHT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_SIDE_RIGHT_FRAMES, fill_frames: None,
};

const TERRAIN_BOTTOM_FRAMES: &[&[u8]] = &[b"\x00\x40\x00\x02\x00\x00\x67\x98"];
pub static TERRAIN_BOTTOM: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_BOTTOM_FRAMES, fill_frames: None,
};
const TERRAIN_BOTTOM2_FRAMES: &[&[u8]] = &[b"\x00\x00\x00\x20\x00\x00\xc5\x3a"];
pub static TERRAIN_BOTTOM2: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_BOTTOM2_FRAMES, fill_frames: None,
};

const TERRAIN_BOTTOM_LEFT_FRAMES: &[&[u8]] = &[b"\x20\x20\x20\x12\x10\x08\x08\x07"];
pub static TERRAIN_BOTTOM_LEFT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_BOTTOM_LEFT_FRAMES, fill_frames: None,
};

const TERRAIN_BOTTOM_RIGHT_FRAMES: &[&[u8]] = &[b"\x04\x14\x04\x08\x08\x10\x10\xe0"];
pub static TERRAIN_BOTTOM_RIGHT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_BOTTOM_RIGHT_FRAMES, fill_frames: None,
};

const TERRAIN_TOP_BOTTOM_FRAMES: &[&[u8]] = &[b"\xff\x00\x77\x77\x00\xa1\x0c\xf3"];
pub static TERRAIN_TOP_BOTTOM: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_BOTTOM_FRAMES, fill_frames: None,
};

const TERRAIN_TOP_LEFT_BOTTOM_FRAMES: &[&[u8]] = &[b"\x7f\x80\xb7\xb7\x80\x88\x60\x1f"];
pub static TERRAIN_TOP_LEFT_BOTTOM: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_LEFT_BOTTOM_FRAMES, fill_frames: None,
};

const TERRAIN_TOP_RIGHT_BOTTOM_FRAMES: &[&[u8]] = &[b"\xfe\x01\x75\x75\x01\xa1\x06\xf8"];
pub static TERRAIN_TOP_RIGHT_BOTTOM: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_RIGHT_BOTTOM_FRAMES, fill_frames: None,
};

const TERRAIN_LEFT_RIGHT_FRAMES: &[&[u8]] = &[b"\x81\x81\x81\x41\x42\x81\x81\x81"];
pub static TERRAIN_LEFT_RIGHT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_LEFT_RIGHT_FRAMES, fill_frames: None,
};

const TERRAIN_LEFT_RIGHT_BOTTOM_FRAMES: &[&[u8]] = &[b"\x81\x81\x81\x41\x41\x42\x22\x1c"];
pub static TERRAIN_LEFT_RIGHT_BOTTOM: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_LEFT_RIGHT_BOTTOM_FRAMES, fill_frames: None,
};

const TERRAIN_TOP_LEFT_RIGHT_FRAMES: &[&[u8]] = &[b"\xff\x81\xb5\xb5\x81\xa1\x85\x81"];
pub static TERRAIN_TOP_LEFT_RIGHT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_LEFT_RIGHT_FRAMES, fill_frames: None,
};

const TERRAIN_TOP_LEFT_BOTTOM_RIGHT_FRAMES: &[&[u8]] = &[b"\xff\x81\xb5\xa5\x81\x92\x42\x3c"];
pub static TERRAIN_TOP_LEFT_BOTTOM_RIGHT: Sprite = Sprite {
    width: 8, height: 8, frames: TERRAIN_TOP_LEFT_BOTTOM_RIGHT_FRAMES, fill_frames: None,
};

// TERRAIN_TILES[tile_type][variant_idx]
pub static TERRAIN_TILES: &[&[&'static Sprite]] = &[
    &[&TERRAIN_TOP, &TERRAIN_TOP2],
    &[&TERRAIN_TOP_LEFT],
    &[&TERRAIN_TOP_RIGHT],
    &[&TERRAIN_SIDE_LEFT],
    &[&TERRAIN_SIDE_RIGHT],
    &[&TERRAIN_BOTTOM, &TERRAIN_BOTTOM2],
    &[&TERRAIN_BOTTOM_LEFT],
    &[&TERRAIN_BOTTOM_RIGHT],
    &[&TERRAIN_TOP_BOTTOM],
    &[&TERRAIN_TOP_LEFT_BOTTOM],
    &[&TERRAIN_TOP_RIGHT_BOTTOM],
    &[&TERRAIN_LEFT_RIGHT],
    &[&TERRAIN_LEFT_RIGHT_BOTTOM],
    &[&TERRAIN_TOP_LEFT_RIGHT],
    &[&TERRAIN_TOP_LEFT_BOTTOM_RIGHT],
];

const PLATFORMER_CHECKPOINT_DOWN_FRAMES: &[&[u8]] = &[
    b"\x1e\x00\x00\x21\x00\x00\x2d\x7f\xff\x2d\x7f\xff\x21\x00\x00\x21\x00\x7e\x61\x87\xfc\xff\xcf\xf8",
];
pub static PLATFORMER_CHECKPOINT_DOWN: Sprite = Sprite {
    width: 24, height: 8, frames: PLATFORMER_CHECKPOINT_DOWN_FRAMES, fill_frames: None,
};

const PLATFORMER_CHECKPOINT_UP_FRAMES: &[&[u8]] = &[
    b"\x0d\x80\x0d\xc0\x0d\xe0\x0d\xf0\x0d\xf8\x0d\xfc\x0c\x7e\x0c\x1e\x0c\x00\x0c\x00\x0c\x00\x0c\x00\x0c\x00\x0c\x00\x0c\x00\x00\x00\x1e\x00\x21\x00\x2d\x00\x2d\x00\x21\x00\x21\x00\x61\x80\xff\xc0",
];
pub static PLATFORMER_CHECKPOINT_UP: Sprite = Sprite {
    width: 15, height: 24, frames: PLATFORMER_CHECKPOINT_UP_FRAMES, fill_frames: None,
};

const PLATFORMER_DOOR_LOCKED_FRAMES: &[&[u8]] = &[
    b"\x33\xcc\x78\x1e\xff\xff\xe0\x07\xc0\x03\xd2\x4b\xdf\xfb\xd2\x4b\xd2\x4b\xd2\x4b\xd2\x4b\xd2\x4b\xd2\x4b\xd2\x4b\xd2\x4b\xd2\x4b\xdf\xfb\xd2\x4b\xd2\x4b",
];
pub static PLATFORMER_DOOR_LOCKED: Sprite = Sprite {
    width: 16, height: 19, frames: PLATFORMER_DOOR_LOCKED_FRAMES, fill_frames: None,
};

const PLATFORMER_DOOR_FRAMES: &[&[u8]] = &[
    b"\x33\xcc\x78\x1e\xff\xff\xe0\x07\xc0\x03\xd0\x03\xd8\x03\xdc\x03\xde\x03\xdc\x03\xde\x83\xdc\x43\xde\x83\xdc\x43\xde\x83\xdc\x43\xde\x83\xdc\x43\xde\x83",
];
pub static PLATFORMER_DOOR: Sprite = Sprite {
    width: 16, height: 19, frames: PLATFORMER_DOOR_FRAMES, fill_frames: None,
};

const PLATFORMER_BG_TILE_1_FRAMES: &[&[u8]] = &[b"\x00\x40\x00\x00\x04\x00\x20\x00"];
pub static PLATFORMER_BG_TILE_1: Sprite = Sprite {
    width: 8, height: 8, frames: PLATFORMER_BG_TILE_1_FRAMES, fill_frames: None,
};
const PLATFORMER_BG_TILE_2_FRAMES: &[&[u8]] = &[b"\x00\x20\x04\x00\x40\x00\x04\x00"];
pub static PLATFORMER_BG_TILE_2: Sprite = Sprite {
    width: 8, height: 8, frames: PLATFORMER_BG_TILE_2_FRAMES, fill_frames: None,
};

// PLATFORMER_BG_TILES[group_idx][variant_idx]
pub static PLATFORMER_BG_TILES: &[&[&'static Sprite]] = &[
    &[&PLATFORMER_BG_TILE_1, &PLATFORMER_BG_TILE_2],
];
