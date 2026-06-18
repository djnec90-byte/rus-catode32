use embedded_graphics::prelude::Point;

use crate::render::{Renderer, Sprite, SpriteOpts};

/// Speech bubble frame sprite (17x17). One frame, no fill layer.
const SPEECH_BUBBLE_FRAMES: &[&[u8]] = &[
    b"\x3f\xfe\x00\x7f\xff\x00\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\x7f\xff\x00\x3f\xfe\x00\x0f\xfc\x00\x00\xfe\x00\x00\x1f\x00\x00\x07\x80",
];

pub const SPEECH_BUBBLE: Sprite = Sprite {
    width: 17,
    height: 17,
    frames: SPEECH_BUBBLE_FRAMES,
    fill_frames: None,
};

/// 9x9 content sprites drawn inside the bubble, inverted.
pub const BUBBLE_W: u16 = 9;
pub const BUBBLE_H: u16 = 9;

const HEART: &[u8] =
    b"\x36\x00\x7f\x00\xff\x80\xff\x80\xff\x80\x7f\x00\x3e\x00\x1c\x00\x08\x00";
const QUESTION: &[u8] =
    b"\x3e\x00\x7f\x00\x73\x00\x67\x00\x0e\x00\x1c\x00\x00\x00\x1c\x00\x1c\x00";
const EXCLAIM: &[u8] =
    b"\x1c\x00\x3c\x00\x3c\x00\x3c\x00\x3c\x00\x18\x00\x00\x00\x0c\x00\x0c\x00";
const NOTE: &[u8] = b"\x0c\x00\x0e\x00\x0b\x00\x08\x00\x08\x00\x08\x00\x38\x00\x78\x00\x30\x00";
const STAR: &[u8] = b"\x08\x00\x1c\x00\x1c\x00\xff\x80\x7f\x00\x3e\x00\x3e\x00\x63\x00\x41\x00";
const HUNGER: &[u8] =
    b"\x10\x80\x7d\x80\xfd\x00\xbf\x00\xff\x00\xff\x00\x7d\x80\x38\x80\x04\x00";
const DISCOMFORT: &[u8] =
    b"\x00\x00\x30\x00\x78\x00\x7e\x00\xff\x80\xff\x80\xff\x00\x76\x00\x00\x00";
const MINICAT: &[u8] =
    b"\x08\x80\x8f\x80\x8a\x80\x8f\x80\xff\x00\xfe\x00\xfe\x00\xaa\x00\xaa\x00";
const MINIGAME: &[u8] =
    b"\x7f\x00\x41\x00\x5d\x00\x5d\x00\x41\x00\x45\x00\x51\x00\x41\x00\x7e\x00";
const HOME: &[u8] = b"\x1c\x00\x3e\x00\x7f\x00\xff\x80\x41\x00\x55\x00\x51\x00\x51\x00\x7f\x00";
const HOT: &[u8] = b"\x08\x00\x41\x00\x1c\x00\x3e\x00\xbe\x80\x3e\x00\x1c\x00\x41\x00\x08\x00";
const WET: &[u8] = b"\x08\x00\x1c\x00\x3e\x00\x3e\x00\x7f\x00\x6f\x00\x55\x00\x2a\x00\x1c\x00";
const COLD: &[u8] = b"\x1c\x00\xc9\x80\xeb\x80\x3e\x00\x08\x00\x3e\x00\xeb\x80\xc9\x80\x1c\x00";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BubbleIcon {
    Heart,
    Question,
    Exclaim,
    Note,
    Star,
    Hunger,
    Discomfort,
    Bored,   // minigame sprite
    Lonely,  // minicat sprite
    Home,
    Hot,
    Wet,
    Cold,
}

impl BubbleIcon {
    fn data(self) -> &'static [u8] {
        match self {
            BubbleIcon::Heart => HEART,
            BubbleIcon::Question => QUESTION,
            BubbleIcon::Exclaim => EXCLAIM,
            BubbleIcon::Note => NOTE,
            BubbleIcon::Star => STAR,
            BubbleIcon::Hunger => HUNGER,
            BubbleIcon::Discomfort => DISCOMFORT,
            BubbleIcon::Bored => MINIGAME,
            BubbleIcon::Lonely => MINICAT,
            BubbleIcon::Home => HOME,
            BubbleIcon::Hot => HOT,
            BubbleIcon::Wet => WET,
            BubbleIcon::Cold => COLD,
        }
    }

    /// Look up a bubble icon by the Python-side string name. Used by behaviors
    /// that store a `&'static str` hint in `ctx.pending_popup_icon`.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "heart" => BubbleIcon::Heart,
            "question" => BubbleIcon::Question,
            "exclaim" => BubbleIcon::Exclaim,
            "note" => BubbleIcon::Note,
            "star" => BubbleIcon::Star,
            "hunger" => BubbleIcon::Hunger,
            "discomfort" => BubbleIcon::Discomfort,
            "bored" => BubbleIcon::Bored,
            "lonely" => BubbleIcon::Lonely,
            "home" => BubbleIcon::Home,
            "hot" => BubbleIcon::Hot,
            "wet" => BubbleIcon::Wet,
            "cold" => BubbleIcon::Cold,
            _ => return None,
        })
    }
}

/// Draw a speech bubble above a character.
///
/// `char_x`/`char_y` is the character's screen position. `progress` (0.0–1.0)
/// drifts the bubble upward up to 10 pixels. `mirror=true` puts the bubble on
/// the right side of the character (tail pointing right).
pub fn draw_above_char(
    renderer: &mut Renderer,
    icon: BubbleIcon,
    char_x: i32,
    char_y: i32,
    progress: f32,
    mirror: bool,
) {
    const DRIFT: i32 = 10;
    let bubble_y = char_y - 45 - (progress * DRIFT as f32) as i32;
    let bubble_x = if mirror {
        char_x + 15
    } else {
        char_x - SPEECH_BUBBLE.width as i32 - 15
    };

    renderer.draw_sprite(
        &SPEECH_BUBBLE,
        Point::new(bubble_x, bubble_y),
        SpriteOpts {
            mirror_h: mirror,
            ..Default::default()
        },
    );

    renderer.draw_sprite_raw(
        icon.data(),
        BUBBLE_W,
        BUBBLE_H,
        Point::new(bubble_x + 4, bubble_y + 2),
        SpriteOpts {
            invert: true,
            transparent: true,
            transparent_color: true,
            ..Default::default()
        },
    );
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Corner {
    Left,
    Right,
}

/// Draw an upside-down speech bubble in a top corner — represents a sound
/// heard from a nearby cat's device.
pub fn draw_heard(renderer: &mut Renderer, icon: BubbleIcon, corner: Corner, y_offset: i32) {
    let (bubble_x, mirror_h) = match corner {
        Corner::Left => (2, true),
        Corner::Right => (128 - SPEECH_BUBBLE.width as i32 - 2, false),
    };

    renderer.draw_sprite(
        &SPEECH_BUBBLE,
        Point::new(bubble_x, y_offset),
        SpriteOpts {
            mirror_v: true,
            mirror_h,
            ..Default::default()
        },
    );

    // Body occupies the lower ~13 rows once flipped; offset down past the tail.
    renderer.draw_sprite_raw(
        icon.data(),
        BUBBLE_W,
        BUBBLE_H,
        Point::new(bubble_x + 4, y_offset + 5),
        SpriteOpts {
            invert: true,
            transparent: true,
            transparent_color: true,
            ..Default::default()
        },
    );
}
