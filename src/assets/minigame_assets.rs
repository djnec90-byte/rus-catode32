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
