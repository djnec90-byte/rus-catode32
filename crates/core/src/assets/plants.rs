#![allow(dead_code)]
//! Sprites for the plant system (pots and plants).
//!
//! Lookups `pot_sprite()` and `plant_sprite()` return `Option<&'static Sprite>`
//! so callers can short-circuit empty / dead / dormant states.

use crate::t;

use crate::{
    context::{PotSize, SeedKind},
    render::Sprite,
};

/// Growth stages used by the renderer to look up the right sprite, plus
/// the special terminal stages.
///
/// `*_Dead` variants only exist for art / debug-preview purposes. The live
/// game keeps a single terminal `Dead` state.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlantStage {
    EmptyPot,
    Seedling,
    Young,
    Growing,
    Mature,
    Thriving,
    SeedlingWilted,
    YoungWilted,
    GrowingWilted,
    MatureWilted,
    ThrivingWilted,
    SeedlingDead,
    YoungDead,
    GrowingDead,
    MatureDead,
    ThrivingDead,
    Dead,
    Dormant,
}

impl PlantStage {
    /// Strip the `_wilted` / `_dead` suffix to recover the underlying stage.
    pub fn base(self) -> Self {
        match self {
            PlantStage::SeedlingWilted | PlantStage::SeedlingDead => PlantStage::Seedling,
            PlantStage::YoungWilted | PlantStage::YoungDead => PlantStage::Young,
            PlantStage::GrowingWilted | PlantStage::GrowingDead => PlantStage::Growing,
            PlantStage::MatureWilted | PlantStage::MatureDead => PlantStage::Mature,
            PlantStage::ThrivingWilted | PlantStage::ThrivingDead => PlantStage::Thriving,
            other => other,
        }
    }

    pub fn is_wilted(self) -> bool {
        matches!(
            self,
            PlantStage::SeedlingWilted
                | PlantStage::YoungWilted
                | PlantStage::GrowingWilted
                | PlantStage::MatureWilted
                | PlantStage::ThrivingWilted
        )
    }

    /// Any dead variant: per-stage death art or the generic terminal `Dead`.
    pub fn is_dead(self) -> bool {
        matches!(
            self,
            PlantStage::Dead
                | PlantStage::SeedlingDead
                | PlantStage::YoungDead
                | PlantStage::GrowingDead
                | PlantStage::MatureDead
                | PlantStage::ThrivingDead
        )
    }

    /// Add the `_wilted` suffix to a healthy base stage. Returns `self` for
    /// stages that don't have a wilted variant (e.g. EmptyPot).
    pub fn wilted_variant(self) -> Self {
        match self {
            PlantStage::Seedling => PlantStage::SeedlingWilted,
            PlantStage::Young => PlantStage::YoungWilted,
            PlantStage::Growing => PlantStage::GrowingWilted,
            PlantStage::Mature => PlantStage::MatureWilted,
            PlantStage::Thriving => PlantStage::ThrivingWilted,
            other => other,
        }
    }

    /// Per-stage dead-art variant. Only used by the debug viewer. The live
    /// game collapses death to the single terminal `Dead` state.
    pub fn dead_variant(self) -> Self {
        match self {
            PlantStage::Seedling | PlantStage::SeedlingWilted => PlantStage::SeedlingDead,
            PlantStage::Young | PlantStage::YoungWilted => PlantStage::YoungDead,
            PlantStage::Growing | PlantStage::GrowingWilted => PlantStage::GrowingDead,
            PlantStage::Mature | PlantStage::MatureWilted => PlantStage::MatureDead,
            PlantStage::Thriving | PlantStage::ThrivingWilted => PlantStage::ThrivingDead,
            other => other,
        }
    }

    /// Display label used by inspect lines and the debug scene.
    pub fn label(self) -> &'static str {
        match self {
            PlantStage::EmptyPot => "Empty pot",
            PlantStage::Seedling => "Seedling",
            PlantStage::SeedlingWilted => "Wilting seedling",
            PlantStage::SeedlingDead => "Dead seedling",
            PlantStage::Young => "Young",
            PlantStage::YoungWilted => "Wilting",
            PlantStage::YoungDead => "Dead",
            PlantStage::Growing => "Growing",
            PlantStage::GrowingWilted => "Wilting",
            PlantStage::GrowingDead => "Dead",
            PlantStage::Mature => "Mature",
            PlantStage::MatureWilted => "Wilting",
            PlantStage::MatureDead => "Dead",
            PlantStage::Thriving => "Thriving",
            PlantStage::ThrivingWilted => "Wilting",
            PlantStage::ThrivingDead => "Dead",
            PlantStage::Dead => "Dead",
            PlantStage::Dormant => "Dormant",
        }
    }
}

/// Pot size used by the renderer. Extends `PotSize` with the synthetic
/// `Ground` value (which has no sprite).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PotKind {
    Small,
    Medium,
    Large,
    Planter,
    Ground,
}

impl PotKind {
    pub fn from_pot_size(p: PotSize) -> Self {
        match p {
            PotSize::Small => PotKind::Small,
            PotSize::Medium => PotKind::Medium,
            PotSize::Large => PotKind::Large,
            PotSize::Planter => PotKind::Planter,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PotKind::Small => t!("Small"),
            PotKind::Medium => t!("Medium"),
            PotKind::Large => t!("Large"),
            PotKind::Planter => t!("Planter"),
            PotKind::Ground => "Ground",
        }
    }
}

/// Look up a pot sprite by kind. Returns None for ground plants.
pub fn pot_sprite(kind: PotKind) -> Option<&'static Sprite> {
    match kind {
        PotKind::Small => Some(&PLANTER1),
        PotKind::Medium => Some(&POT_MEDIUM),
        PotKind::Large => Some(&POT_LARGE),
        PotKind::Planter => Some(&PLANTER_SMALL_1),
        PotKind::Ground => None,
    }
}

/// Look up a plant sprite by (seed type, stage). Returns None for stages that
/// intentionally render nothing (EmptyPot, generic terminal Dead, Dormant).
/// The per-stage `*_Dead` variants return real corpse art so a plant that
/// died at e.g. Mature renders `MATURE_DEAD` rather than disappearing.
pub fn plant_sprite(seed: SeedKind, stage: PlantStage) -> Option<&'static Sprite> {
    match stage {
        PlantStage::EmptyPot | PlantStage::Dead | PlantStage::Dormant => None,
        _ => Some(match seed {
            SeedKind::CatGrass => match stage {
                PlantStage::Seedling => &GRASS_SEEDLING,
                PlantStage::SeedlingWilted => &GRASS_SEEDLING_WILTED,
                PlantStage::SeedlingDead => &GRASS_SEEDLING_DEAD,
                PlantStage::Young => &GRASS_YOUNG,
                PlantStage::YoungWilted => &GRASS_YOUNG_WILTED,
                PlantStage::YoungDead => &GRASS_YOUNG_DEAD,
                PlantStage::Growing => &GRASS_GROWING,
                PlantStage::GrowingWilted => &GRASS_GROWING_WILTED,
                PlantStage::GrowingDead => &GRASS_GROWING_DEAD,
                PlantStage::Mature => &GRASS_MATURE,
                PlantStage::MatureWilted => &GRASS_MATURE_WILTED,
                PlantStage::MatureDead => &GRASS_MATURE_DEAD,
                PlantStage::Thriving => &GRASS_THRIVING,
                PlantStage::ThrivingWilted => &GRASS_THRIVING_WILTED,
                PlantStage::ThrivingDead => &GRASS_THRIVING_DEAD,
                _ => return None,
            },
            SeedKind::Freesia => match stage {
                PlantStage::Seedling => &PLANT_SEEDLING,
                PlantStage::SeedlingWilted => &PLANT_SEEDLING_WILTED,
                PlantStage::SeedlingDead => &PLANT_SEEDLING_DEAD,
                PlantStage::Young => &FREESIA_YOUNG,
                PlantStage::YoungWilted => &FREESIA_YOUNG_WILTED,
                PlantStage::YoungDead => &FREESIA_YOUNG_DEAD,
                PlantStage::Growing => &FREESIA_GROWING,
                PlantStage::GrowingWilted => &FREESIA_GROWING_WILTED,
                PlantStage::GrowingDead => &FREESIA_GROWING_DEAD,
                PlantStage::Mature => &FREESIA_MATURE,
                PlantStage::MatureWilted => &FREESIA_MATURE_WILTED,
                PlantStage::MatureDead => &FREESIA_MATURE_DEAD,
                PlantStage::Thriving => &FREESIA_THRIVING,
                PlantStage::ThrivingWilted => &FREESIA_THRIVING_WILTED,
                PlantStage::ThrivingDead => &FREESIA_THRIVING_DEAD,
                _ => return None,
            },
            SeedKind::Sunflower => match stage {
                PlantStage::Seedling => &PLANT_SEEDLING,
                PlantStage::SeedlingWilted => &PLANT_SEEDLING_WILTED,
                PlantStage::SeedlingDead => &PLANT_SEEDLING_DEAD,
                PlantStage::Young => &SUNFLOWER_YOUNG,
                PlantStage::YoungWilted => &SUNFLOWER_YOUNG_WILTED,
                PlantStage::YoungDead => &SUNFLOWER_YOUNG_DEAD,
                PlantStage::Growing => &SUNFLOWER_GROWING,
                PlantStage::GrowingWilted => &SUNFLOWER_GROWING_WILTED,
                PlantStage::GrowingDead => &SUNFLOWER_GROWING_DEAD,
                PlantStage::Mature => &SUNFLOWER_MATURE,
                PlantStage::MatureWilted => &SUNFLOWER_MATURE_WILTED,
                PlantStage::MatureDead => &SUNFLOWER_MATURE_DEAD,
                PlantStage::Thriving => &SUNFLOWER_THRIVING,
                PlantStage::ThrivingWilted => &SUNFLOWER_THRIVING_WILTED,
                PlantStage::ThrivingDead => &SUNFLOWER_THRIVING_DEAD,
                _ => return None,
            },
            SeedKind::Rose => match stage {
                PlantStage::Seedling => &PLANT_SEEDLING,
                PlantStage::SeedlingWilted => &PLANT_SEEDLING_WILTED,
                PlantStage::SeedlingDead => &PLANT_SEEDLING_DEAD,
                PlantStage::Young => &ROSE_YOUNG,
                PlantStage::YoungWilted => &ROSE_YOUNG_WILTED,
                PlantStage::YoungDead => &ROSE_YOUNG_DEAD,
                PlantStage::Growing => &ROSE_GROWING,
                PlantStage::GrowingWilted => &ROSE_GROWING_WILTED,
                PlantStage::GrowingDead => &ROSE_GROWING_DEAD,
                PlantStage::Mature => &ROSE_MATURE,
                PlantStage::MatureWilted => &ROSE_MATURE_WILTED,
                PlantStage::MatureDead => &ROSE_MATURE_DEAD,
                PlantStage::Thriving => &ROSE_THRIVING,
                PlantStage::ThrivingWilted => &ROSE_THRIVING_WILTED,
                PlantStage::ThrivingDead => &ROSE_THRIVING_DEAD,
                _ => return None,
            },
        }),
    }
}

// ---------------------------------------------------------------------------
// Pot sprites
// ---------------------------------------------------------------------------

const PLANTER1_FRAMES: &[&[u8]] = &[
    b"\xff\xf8\xff\xf8\x40\x10\x40\x10\x20\x20\x20\x20\x10\x40\x1f\xc0\x3f\xe0",
];
const PLANTER1_FILL_FRAMES: &[&[u8]] = &[
    b"\xff\xf8\xff\xf8\x7f\xf0\x7f\xf0\x3f\xe0\x3f\xe0\x1f\xc0\x1f\xc0\x3f\xe0",
];
pub static PLANTER1: Sprite = Sprite {
    width: 13,
    height: 9,
    frames: PLANTER1_FRAMES,
    fill_frames: Some(PLANTER1_FILL_FRAMES),
};

const POT_MEDIUM_FRAMES: &[&[u8]] = &[
    b"\xff\xff\x80\x80\x00\x80\x80\x00\x80\xd5\x55\x80\xaa\xaa\x80\x80\x00\x80\x80\x00\x80\x80\x00\x80\x40\x01\x00\x40\x01\x00\x20\x02\x00\x1f\xfc\x00",
];
const POT_MEDIUM_FILL_FRAMES: &[&[u8]] = &[
    b"\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\xff\xff\x80\x7f\xff\x00\x7f\xff\x00\x3f\xfe\x00\x1f\xfc\x00",
];
pub static POT_MEDIUM: Sprite = Sprite {
    width: 17,
    height: 12,
    frames: POT_MEDIUM_FRAMES,
    fill_frames: Some(POT_MEDIUM_FILL_FRAMES),
};

const POT_LARGE_FRAMES: &[&[u8]] = &[
    b"\x7f\xff\xf0\x80\x00\x08\x80\x00\x08\xff\xff\xf8\x40\x00\x10\x5d\x75\xd0\x6b\xae\xb0\x40\x00\x10\x40\x00\x10\x40\x00\x10\x40\x00\x10\x40\x00\x10\x20\x00\x20\x20\x00\x20\x1a\xaa\xc0\x0f\xff\x80",
];
const POT_LARGE_FILL_FRAMES: &[&[u8]] = &[
    b"\x7f\xff\xf0\xff\xff\xf8\xff\xff\xf8\xff\xff\xf8\x7f\xff\xf0\x7f\xff\xf0\x7f\xff\xf0\x7f\xff\xf0\x7f\xff\xf0\x7f\xff\xf0\x7f\xff\xf0\x7f\xff\xf0\x3f\xff\xe0\x3f\xff\xe0\x1f\xff\xc0\x0f\xff\x80",
];
pub static POT_LARGE: Sprite = Sprite {
    width: 21,
    height: 16,
    frames: POT_LARGE_FRAMES,
    fill_frames: Some(POT_LARGE_FILL_FRAMES),
};

const PLANTER_SMALL_1_FRAMES: &[&[u8]] = &[
    b"\x7f\xff\xff\xf0\x80\x00\x00\x08\x80\x00\x00\x08\xff\xff\xff\xf8\x55\x55\x55\x50\x62\x22\x22\x30\x40\x00\x00\x10\x40\x00\x00\x10\x40\x00\x00\x10\x40\x00\x00\x10\x2a\xaa\xaa\xa0\x1f\xff\xff\xc0",
];
const PLANTER_SMALL_1_FILL_FRAMES: &[&[u8]] = &[
    b"\x7f\xff\xff\xf0\xff\xff\xff\xf8\xff\xff\xff\xf8\xff\xff\xff\xf8\x7f\xff\xff\xf0\x7f\xff\xff\xf0\x7f\xff\xff\xf0\x7f\xff\xff\xf0\x7f\xff\xff\xf0\x7f\xff\xff\xf0\x3f\xff\xff\xe0\x1f\xff\xff\xc0",
];
pub static PLANTER_SMALL_1: Sprite = Sprite {
    width: 29,
    height: 12,
    frames: PLANTER_SMALL_1_FRAMES,
    fill_frames: Some(PLANTER_SMALL_1_FILL_FRAMES),
};

/// Height to add below a ground plant so it sits on the surface visually.
pub const GROUND_PLANT_OFFSET: i32 = 0;

// ---------------------------------------------------------------------------
// Generic plant sprites
// ---------------------------------------------------------------------------

const PLANT1_FRAMES: &[&[u8]] = &[
    b"\xc0\x00\xa0\x00\xd0\x70\x70\xb0\x31\x60\x09\xc0\x0a\x00\x04\x00\x64\x00\x54\x1c\x6a\x2c\x3a\x58\x06\x70\x02\x80\x03\x00\x02\x00\x02\x00",
];
pub static PLANT1: Sprite = Sprite {
    width: 14,
    height: 17,
    frames: PLANT1_FRAMES,
    fill_frames: None,
};

const PLANT2_FRAMES: &[&[u8]] = &[
    b"\x02\x10\x00\x01\x28\x00\x02\x90\x40\x41\x28\xa0\xa2\x91\x40\x51\x2a\x80\x2a\x91\x00\x19\x2a\x00\x05\x12\x00\x05\x12\x00",
];
pub static PLANT2: Sprite = Sprite {
    width: 19,
    height: 10,
    frames: PLANT2_FRAMES,
    fill_frames: None,
};

const PLANT6_FRAMES: &[&[u8]] = &[
    b"\x00\x10\x00\x00\x28\x00\x00\x10\x00\x02\x28\x00\x01\x10\x00\x02\xa8\x20\x01\x10\x50\x02\xa8\xa0\x01\x11\x40\x42\x92\x80\xa1\x15\x00\x51\x12\x00\x29\x15\x00\x19\x12\x00\x04\x92\x00\x04\x92\x00\x04\x92\x00\x04\x92\x00\x02\x92\x00\x02\x92\x00\x02\x92\x00\x02\x92\x00\x02\x92\x00",
];
pub static PLANT6: Sprite = Sprite {
    width: 20,
    height: 23,
    frames: PLANT6_FRAMES,
    fill_frames: None,
};

// Shared seedling sprites (used by multiple flower types at the seedling stage)
const PLANT_SEEDLING_FRAMES: &[&[u8]] = &[
    b"\x60\x00\xf0\x00\x77\x00\x3f\x80\x0b\x00\x08\x00\x08\x00",
];
pub static PLANT_SEEDLING: Sprite = Sprite {
    width: 9,
    height: 7,
    frames: PLANT_SEEDLING_FRAMES,
    fill_frames: None,
};

const PLANT_SEEDLING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x30\x00\x7f\x00\xeb\x80\x49\x80",
];
pub static PLANT_SEEDLING_WILTED: Sprite = Sprite {
    width: 9,
    height: 4,
    frames: PLANT_SEEDLING_WILTED_FRAMES,
    fill_frames: None,
};

const PLANT_SEEDLING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x10\x68\xa0",
];
pub static PLANT_SEEDLING_DEAD: Sprite = Sprite {
    width: 5,
    height: 3,
    frames: PLANT_SEEDLING_DEAD_FRAMES,
    fill_frames: None,
};

// ---------------------------------------------------------------------------
// Sunflower sprites: all stages and health states
// ---------------------------------------------------------------------------

const SUNFLOWER_YOUNG_FRAMES: &[&[u8]] = &[
    b"\x02\x00\xc7\x00\xa2\x00\xd2\x70\x74\xb0\x35\x60\x0d\xc0\x06\x00\x04\x00\x64\x1c\x52\x2c\x6a\x58\x3a\x70\x06\x80\x03\x00\x02\x00\x02\x00",
];
pub static SUNFLOWER_YOUNG: Sprite = Sprite {
    width: 14,
    height: 17,
    frames: SUNFLOWER_YOUNG_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_YOUNG_WILTED_FRAMES: &[&[u8]] = &[
    b"\x01\x00\x03\x80\x05\x00\x04\x00\x04\x00\x3c\x00\x57\xc0\xb5\xa0\xe4\xd0\xc2\x70\x02\x00\x02\x00\x3e\x00\x6b\xe0\x52\x58\x62\x38",
];
pub static SUNFLOWER_YOUNG_WILTED: Sprite = Sprite {
    width: 13,
    height: 16,
    frames: SUNFLOWER_YOUNG_WILTED_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_YOUNG_DEAD_FRAMES: &[&[u8]] = &[
    b"\x1c\x28\x20\x60\xb8\x24\x20\x10\x10\x10\x30\x5c\x12\x10",
];
pub static SUNFLOWER_YOUNG_DEAD: Sprite = Sprite {
    width: 7,
    height: 14,
    frames: SUNFLOWER_YOUNG_DEAD_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_GROWING_FRAMES: &[&[u8]] = &[
    b"\x07\x00\x0f\x80\x07\x00\xc2\x00\xa2\x00\xd2\x70\x74\xb0\x35\x60\x0d\xc0\x06\x00\x04\x00\x64\x1c\x52\x2c\x6a\x58\x3a\x70\x06\x80\x03\x00\xc2\x00\xa2\x1c\xd2\x2c\x72\x58\x0a\x70\x06\x80\x03\x00\x02\x00\x02\x00\x02\x00",
];
pub static SUNFLOWER_GROWING: Sprite = Sprite {
    width: 14,
    height: 27,
    frames: SUNFLOWER_GROWING_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_GROWING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x01\xc0\x03\xe0\x01\xc0\x02\x00\x02\x00\x04\x00\x04\x00\x07\xc0\x7d\x60\xd4\xb0\xa4\x70\xc2\x00\x02\x00\x02\x1c\x3e\x2c\x6b\xf0\x52\x00\x62\x00\x02\x00\xc2\x1c\xa2\x2c\xd6\x58\x7b\xf0\x02\x00\x02\x00\x02\x00",
];
pub static SUNFLOWER_GROWING_WILTED: Sprite = Sprite {
    width: 14,
    height: 26,
    frames: SUNFLOWER_GROWING_WILTED_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_GROWING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x06\x00\x09\x00\x0a\x80\x08\x00\x1c\x00\x2a\x00\x0a\x00\x10\x00\x38\x00\x54\x00\x52\x00\x90\x00\x10\x00\x10\x00\x30\x00\x5c\x00\x52\x00\x10\x00\x10\x00",
];
pub static SUNFLOWER_GROWING_DEAD: Sprite = Sprite {
    width: 9,
    height: 19,
    frames: SUNFLOWER_GROWING_DEAD_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_MATURE_FRAMES: &[&[u8]] = &[
    b"\x02\x80\x0b\xa0\x05\x40\x0f\xe0\x06\xc0\xcb\xa0\xa2\x00\xd4\x70\x74\xb0\x35\x60\x0d\xc0\x06\x00\x04\x1c\x64\x2c\x52\x58\x6a\x70\x3a\x80\x07\x00\xc2\x00\xa2\x1c\xd2\x2c\x72\x58\x0a\x70\x06\x80\xc2\x80\xa3\x1c\xd2\x2c\x72\x58\x0a\x70\x06\x80\x07\x00\x02\x00",
];
pub static SUNFLOWER_MATURE: Sprite = Sprite {
    width: 14,
    height: 32,
    frames: SUNFLOWER_MATURE_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_MATURE_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\xe8\x01\xd0\x03\xf8\x01\xb0\x02\xe8\x04\x40\x04\x00\x3e\x00\x75\xc0\xd5\x60\xa4\xb0\x42\x70\x02\x00\x02\xf0\x3f\x58\x6a\x2c\x52\x18\x62\x00\x02\x00\x7a\x18\x86\x70\x02\x80\x43\x00\xa2\x1c\xd2\x2c\x72\x58\x0a\x70\x07\x80\x02\x00",
];
pub static SUNFLOWER_MATURE_WILTED: Sprite = Sprite {
    width: 14,
    height: 29,
    frames: SUNFLOWER_MATURE_WILTED_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_MATURE_DEAD_FRAMES: &[&[u8]] = &[
    b"\x07\x00\x0d\x80\x17\x00\x22\x00\x20\x00\x70\x00\xae\x00\xa3\x00\x20\x00\x10\x00\x10\x00\x17\x00\x38\x00\x50\x00\x90\x00\x10\x00\x10\x00\x10\x00\x30\x00\x54\x00\x1a\x00\x10\x00\x10\x00\x10\x00\x10\x00\x38\x00\x50\x00",
];
pub static SUNFLOWER_MATURE_DEAD: Sprite = Sprite {
    width: 9,
    height: 27,
    frames: SUNFLOWER_MATURE_DEAD_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_THRIVING_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x01\x00\x0b\xa0\x06\xc0\x0d\x60\x1b\xb0\x0d\x60\x06\xc0\x4f\xa0\xa5\x00\xd4\x00\x74\x20\x34\x70\x0c\xb0\x05\x60\x07\xc0\x64\x00\x52\x38\x6a\x58\x3a\xb0\x06\xe0\xc3\x00\xa2\x18\xd2\x2c\x72\x58\x0a\x70\x06\x80\xc2\x80\xa3\x1c\xd2\x2c\x72\x58\x0a\x70\x06\x80\x07\x00\x02\x00",
];
pub static SUNFLOWER_THRIVING: Sprite = Sprite {
    width: 14,
    height: 35,
    frames: SUNFLOWER_THRIVING_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_THRIVING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\xd0\x03\x60\x02\xa0\x0d\xd8\x06\xb0\x03\x60\x05\xd0\x06\x80\x14\x00\x2c\x00\x47\xc0\x05\x60\x04\xb0\x62\x70\xd2\x20\x6a\x00\x3e\xe0\x03\x10\xc2\x00\xa2\x00\xd2\x18\x7a\x2c\x06\x58\x02\xf0\x03\x00\xc2\x00\xa2\x1c\xd2\x2c\x7e\xf8\x07\x00\x02\x00",
];
pub static SUNFLOWER_THRIVING_WILTED: Sprite = Sprite {
    width: 14,
    height: 31,
    frames: SUNFLOWER_THRIVING_WILTED_FRAMES,
    fill_frames: None,
};

const SUNFLOWER_THRIVING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x07\x1f\x2e\x25\x2a\x60\xb8\x24\xa4\x10\x10\x10\x74\x9a\x12\x10\x10\x50\xb0\x94\x5a\x12\x10\x10\x56\xb9\x10",
];
pub static SUNFLOWER_THRIVING_DEAD: Sprite = Sprite {
    width: 8,
    height: 27,
    frames: SUNFLOWER_THRIVING_DEAD_FRAMES,
    fill_frames: None,
};

// ---------------------------------------------------------------------------
// Cat grass sprites: all stages and health states
// ---------------------------------------------------------------------------

const GRASS_SEEDLING_FRAMES: &[&[u8]] = &[
    b"\x20\xa0\xa8",
];
pub static GRASS_SEEDLING: Sprite = Sprite {
    width: 5,
    height: 3,
    frames: GRASS_SEEDLING_FRAMES,
    fill_frames: None,
};

const GRASS_SEEDLING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\x8c\x50",
];
pub static GRASS_SEEDLING_WILTED: Sprite = Sprite {
    width: 7,
    height: 3,
    frames: GRASS_SEEDLING_WILTED_FRAMES,
    fill_frames: None,
};

const GRASS_SEEDLING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x58",
];
pub static GRASS_SEEDLING_DEAD: Sprite = Sprite {
    width: 7,
    height: 3,
    frames: GRASS_SEEDLING_DEAD_FRAMES,
    fill_frames: None,
};

const GRASS_YOUNG_FRAMES: &[&[u8]] = &[
    b"\x00\x48\x8a\xaa\xaa\xaa",
];
pub static GRASS_YOUNG: Sprite = Sprite {
    width: 7,
    height: 6,
    frames: GRASS_YOUNG_FRAMES,
    fill_frames: None,
};

const GRASS_YOUNG_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x00\x80\x11\x00\x4a\x40\xaa\xa0\x2a\x80",
];
pub static GRASS_YOUNG_WILTED: Sprite = Sprite {
    width: 11,
    height: 6,
    frames: GRASS_YOUNG_WILTED_FRAMES,
    fill_frames: None,
};

const GRASS_YOUNG_DEAD_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x00\x00\x00\x00\x31\x80\x0a\x20\xea\xc0",
];
pub static GRASS_YOUNG_DEAD: Sprite = Sprite {
    width: 11,
    height: 6,
    frames: GRASS_YOUNG_DEAD_FRAMES,
    fill_frames: None,
};

const GRASS_GROWING_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x84\x00\x92\x80\x51\x00\x55\x00\x55\x00\x55\x00\x55\x00\x55\x00",
];
pub static GRASS_GROWING: Sprite = Sprite {
    width: 9,
    height: 9,
    frames: GRASS_GROWING_FRAMES,
    fill_frames: None,
};

const GRASS_GROWING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x12\x00\x49\x18\xa5\x20\x95\x50\x15\x48\x15\x40",
];
pub static GRASS_GROWING_WILTED: Sprite = Sprite {
    width: 13,
    height: 6,
    frames: GRASS_GROWING_WILTED_FRAMES,
    fill_frames: None,
};

const GRASS_GROWING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x1c\x70\x32\x98\xca\xae",
];
pub static GRASS_GROWING_DEAD: Sprite = Sprite {
    width: 15,
    height: 3,
    frames: GRASS_GROWING_DEAD_FRAMES,
    fill_frames: None,
};

const GRASS_MATURE_FRAMES: &[&[u8]] = &[
    b"\x82\x80\x54\xa0\x24\x40\xa8\x40\x6a\xc0\x2a\x40\x2a\x40\xaa\x60\x6a\x40\x2a\x40\x2a\x40",
];
pub static GRASS_MATURE: Sprite = Sprite {
    width: 11,
    height: 11,
    frames: GRASS_MATURE_FRAMES,
    fill_frames: None,
};

const GRASS_MATURE_WILTED_FRAMES: &[&[u8]] = &[
    b"\x18\x30\x04\x44\x42\x8a\xa2\xd0\x92\x98\xaa\x94\x0a\x90\x0a\x90",
];
pub static GRASS_MATURE_WILTED: Sprite = Sprite {
    width: 15,
    height: 8,
    frames: GRASS_MATURE_WILTED_FRAMES,
    fill_frames: None,
};

const GRASS_MATURE_DEAD_FRAMES: &[&[u8]] = &[
    b"\x3c\x3c\xd2\x4a\x2a\x94\xca\x93",
];
pub static GRASS_MATURE_DEAD: Sprite = Sprite {
    width: 16,
    height: 4,
    frames: GRASS_MATURE_DEAD_FRAMES,
    fill_frames: None,
};

const GRASS_THRIVING_FRAMES: &[&[u8]] = &[
    b"\x02\x10\x00\x01\x28\x00\x02\x90\x40\x41\x28\xa0\xa2\x91\x40\x51\x2a\x80\x2a\x91\x00\x19\x2a\x00\x05\x12\x80\x25\x13\x00\x15\x12\x00\x0d\x52\x00\x05\x32\x00\x05\x12\x40\x35\x12\x80\x0d\x13\x00\x05\x12\x00\x05\x12\x00",
];
pub static GRASS_THRIVING: Sprite = Sprite {
    width: 19,
    height: 18,
    frames: GRASS_THRIVING_FRAMES,
    fill_frames: None,
};

const GRASS_THRIVING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x4a\x00\xa5\x24\x52\x4a\x32\x8c\x0a\x92\x3a\xa8\x0a\xa4\x2a\xa0\x5a\xb0\x8a\xa8\x0a\xa0",
];
pub static GRASS_THRIVING_WILTED: Sprite = Sprite {
    width: 15,
    height: 11,
    frames: GRASS_THRIVING_WILTED_FRAMES,
    fill_frames: None,
};

const GRASS_THRIVING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x3e\x0e\x00\x41\x11\x80\xbc\xa4\x00\x46\xab\xa0\xaa\xa9\x40",
];
pub static GRASS_THRIVING_DEAD: Sprite = Sprite {
    width: 19,
    height: 5,
    frames: GRASS_THRIVING_DEAD_FRAMES,
    fill_frames: None,
};

// ---------------------------------------------------------------------------
// Rose sprites: all stages and health states
// ---------------------------------------------------------------------------

const ROSE_YOUNG_FRAMES: &[&[u8]] = &[
    b"\x04\x00\x72\x00\x3a\x00\x1e\x00\x02\x60\x62\xe0\xf3\xc0\x7a\x00\x1e\x00\x02\x00\x04\x00\x04\x00",
];
pub static ROSE_YOUNG: Sprite = Sprite {
    width: 11,
    height: 12,
    frames: ROSE_YOUNG_FRAMES,
    fill_frames: None,
};

const ROSE_YOUNG_WILTED_FRAMES: &[&[u8]] = &[
    b"\x03\x00\x3c\x80\x74\x00\x64\x00\x07\x80\x05\xc0\x7c\xc0\xe4\x00\xc8\x00\x08\x00",
];
pub static ROSE_YOUNG_WILTED: Sprite = Sprite {
    width: 10,
    height: 10,
    frames: ROSE_YOUNG_WILTED_FRAMES,
    fill_frames: None,
};

const ROSE_YOUNG_DEAD_FRAMES: &[&[u8]] = &[
    b"\x2c\x12\x12\x72\x98\x24\x20",
];
pub static ROSE_YOUNG_DEAD: Sprite = Sprite {
    width: 7,
    height: 7,
    frames: ROSE_YOUNG_DEAD_FRAMES,
    fill_frames: None,
};

const ROSE_GROWING_FRAMES: &[&[u8]] = &[
    b"\x0a\xa0\x07\xc0\x07\xc0\x07\xc0\x03\x80\x01\x00\x31\x00\x79\x0c\xfd\x3e\x3b\x7c\x01\x98\x61\x00\xf9\x00\x7f\x78\x0d\xbc\x19\x18\x39\x00\x32\x00\x03\xe0\x02\xf0\x02\x20",
];
pub static ROSE_GROWING: Sprite = Sprite {
    width: 15,
    height: 21,
    frames: ROSE_GROWING_FRAMES,
    fill_frames: None,
};

const ROSE_GROWING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x01\xc0\x06\xc0\x7c\x00\xe4\x00\x07\x80\x3d\xc0\x74\x40\x64\x00\x04\x00\xc7\x00\xf5\x80\x3d\xc0\x64\xc0\x48\x00\x08\x00",
];
pub static ROSE_GROWING_WILTED: Sprite = Sprite {
    width: 10,
    height: 15,
    frames: ROSE_GROWING_WILTED_FRAMES,
    fill_frames: None,
};

const ROSE_GROWING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x06\x29\x51\x59\x14\x74\x94\x10\x20",
];
pub static ROSE_GROWING_DEAD: Sprite = Sprite {
    width: 8,
    height: 9,
    frames: ROSE_GROWING_DEAD_FRAMES,
    fill_frames: None,
};

const ROSE_MATURE_FRAMES: &[&[u8]] = &[
    b"\x02\xa8\x00\x01\x50\x00\x01\xf0\x00\x01\xf0\x00\x00\xe0\x00\x1e\x43\x00\x0f\x5f\xc0\x01\xe7\x80\x00\x40\x00\x78\x41\x80\xfe\x47\xc0\x39\xdf\x80\x00\x63\x00\x60\x40\x00\xf8\x40\x00\x7f\xdf\x80\x0c\x63\xc0\x18\x41\x80\x38\x40\x00\x30\x80\x00\x00\xf8\x00\x00\xbc\x00\x00\x88\x00",
];
pub static ROSE_MATURE: Sprite = Sprite {
    width: 19,
    height: 23,
    frames: ROSE_MATURE_FRAMES,
    fill_frames: None,
};

const ROSE_MATURE_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\x28\x00\x00\x54\x00\x00\x7a\x00\x00\x7c\x00\x00\x3c\x00\x00\x20\x00\x00\x40\x00\x0f\xfc\x00\x1e\x4e\x00\x08\x47\x00\x00\x80\x00\x7f\x80\x00\xf0\xfe\x00\x60\x8f\x00\x00\x87\x80\x7f\x80\x00\xe4\xfc\x00\xcc\x8e\x00\x9c\x87\x00\x18\xf1\x00\x00\xb8\x00\x01\x0c\x00\x01\x00\x00",
];
pub static ROSE_MATURE_WILTED: Sprite = Sprite {
    width: 17,
    height: 23,
    frames: ROSE_MATURE_WILTED_FRAMES,
    fill_frames: None,
};

const ROSE_MATURE_DEAD_FRAMES: &[&[u8]] = &[
    b"\x01\xc0\x02\xe0\x04\x70\x04\x70\x08\xb0\x38\x50\x4c\x20\x4a\x00\x11\x00\x30\x00\x5c\x00\x92\x00\x91\x00\x08\x00\x08\x00\x14\x00\x04\x00",
];
pub static ROSE_MATURE_DEAD: Sprite = Sprite {
    width: 12,
    height: 17,
    frames: ROSE_MATURE_DEAD_FRAMES,
    fill_frames: None,
};

const ROSE_THRIVING_FRAMES: &[&[u8]] = &[
    b"\x01\x50\x00\x06\xac\x00\x03\x58\x00\x01\xf0\x00\x01\xf0\x00\x00\xe0\x40\x18\x40\xa0\x28\x46\xd0\x58\x4e\xe8\xbb\x2c\xf0\x7b\xb9\xf8\xf9\xe3\x00\x08\x25\x80\x04\x39\xc0\x3f\x20\xc0\x78\xec\x00\x60\x39\x80\x02\x23\x80\x03\x27\x00\x78\xe7\xe0\xff\x38\xf0\x02\x20\x60\x06\x30\x00\x0e\x2f\x00\x1c\x47\x80\x00\x41\x00\x00\x40\x00",
];
pub static ROSE_THRIVING: Sprite = Sprite {
    width: 21,
    height: 27,
    frames: ROSE_THRIVING_FRAMES,
    fill_frames: None,
};

const ROSE_THRIVING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\x50\x00\x00\xb0\x00\x01\x70\x00\x02\xf0\x00\x01\xf0\x00\x02\x90\x00\x00\x11\xc0\x00\x0a\x20\x00\x6c\x07\x00\xb8\xfe\x01\x09\x1d\x78\x0e\x9e\xff\xc8\x55\x7c\xb8\x0a\xb9\x0f\x00\x50\x08\x80\x29\xc8\x30\x02\x39\xc0\x00\x4e\x60\x03\x88\x30\x04\x8c\x10\x09\x8b\x00\x0b\x11\x80\x03\x10\xc0\x02\x10\x00",
];
pub static ROSE_THRIVING_WILTED: Sprite = Sprite {
    width: 24,
    height: 25,
    frames: ROSE_THRIVING_WILTED_FRAMES,
    fill_frames: None,
};

const ROSE_THRIVING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x38\x00\x00\x7c\x00\x00\x73\x00\x00\x54\x9e\x00\x20\x67\x00\x00\x47\x00\x03\xc3\x80\x1c\x65\x00\x38\x52\x80\x51\x40\x00\x2a\xe0\x00\x00\x90\x00\x01\x10\x00\x0f\x08\x00\x12\x80\x00\x12\x40\x00\x21\x00\x00\x01\x00\x00",
];
pub static ROSE_THRIVING_DEAD: Sprite = Sprite {
    width: 17,
    height: 18,
    frames: ROSE_THRIVING_DEAD_FRAMES,
    fill_frames: None,
};

// ---------------------------------------------------------------------------
// Freesia sprites: all stages and health states
// ---------------------------------------------------------------------------

const FREESIA_YOUNG_FRAMES: &[&[u8]] = &[
    b"\x08\x00\x1c\x00\x1c\x00\x88\x80\xc9\x80\xeb\x80\x7b\x00\x3e\x00\x1c\x00\x08\x00\x08\x00",
];
pub static FREESIA_YOUNG: Sprite = Sprite {
    width: 9,
    height: 11,
    frames: FREESIA_YOUNG_FRAMES,
    fill_frames: None,
};

const FREESIA_YOUNG_WILTED_FRAMES: &[&[u8]] = &[
    b"\x04\x00\x0e\x00\x0c\x00\x08\x00\x08\x00\xcb\x80\x7f\x00\x3c\x00\x08\x00\x08\x00",
];
pub static FREESIA_YOUNG_WILTED: Sprite = Sprite {
    width: 9,
    height: 10,
    frames: FREESIA_YOUNG_WILTED_FRAMES,
    fill_frames: None,
};

const FREESIA_YOUNG_DEAD_FRAMES: &[&[u8]] = &[
    b"\x10\x38\x58\x40\xe0\x50\x40",
];
pub static FREESIA_YOUNG_DEAD: Sprite = Sprite {
    width: 5,
    height: 7,
    frames: FREESIA_YOUNG_DEAD_FRAMES,
    fill_frames: None,
};

const FREESIA_GROWING_FRAMES: &[&[u8]] = &[
    b"\x04\x00\x0e\x00\x0e\x00\x0e\x20\x84\x20\xc4\x60\xe4\xe0\xf5\xe0\x75\xc0\x7f\x80\x3f\x80\x1e\x00\x04\x00\x04\x00\x04\x00",
];
pub static FREESIA_GROWING: Sprite = Sprite {
    width: 11,
    height: 15,
    frames: FREESIA_GROWING_FRAMES,
    fill_frames: None,
};

const FREESIA_GROWING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x02\x00\x07\x00\x07\x00\x06\x00\x04\x00\x04\x00\x44\xc0\x65\xe0\xff\xe0\xdf\x20\x8e\x20\x84\x20\x04\x00\x04\x00",
];
pub static FREESIA_GROWING_WILTED: Sprite = Sprite {
    width: 11,
    height: 15,
    frames: FREESIA_GROWING_WILTED_FRAMES,
    fill_frames: None,
};

const FREESIA_GROWING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x04\x00\x0a\x00\x13\x00\x13\x80\x11\x00\x10\x00\x54\x00\xba\x00\x12\x00\x10\x00\x10\x00",
];
pub static FREESIA_GROWING_DEAD: Sprite = Sprite {
    width: 9,
    height: 11,
    frames: FREESIA_GROWING_DEAD_FRAMES,
    fill_frames: None,
};

const FREESIA_MATURE_FRAMES: &[&[u8]] = &[
    b"\x07\x00\x07\x00\x0f\x80\x0f\x80\x0f\x80\x07\x00\x22\x00\x32\x00\x1a\x08\x1a\x18\x8e\x38\xe6\x70\x7a\xe0\x3f\xc0\x1f\x80\x0f\x00\x02\x00\x02\x00\x02\x00",
];
pub static FREESIA_MATURE: Sprite = Sprite {
    width: 13,
    height: 19,
    frames: FREESIA_MATURE_FRAMES,
    fill_frames: None,
};

const FREESIA_MATURE_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\x40\x00\xe0\x01\xe0\x01\xe0\x03\xc0\x03\x00\x02\x00\x02\x00\x72\x00\xde\x30\x0e\x70\x1f\xd8\x7f\x88\xe7\x00\xc2\x00\x82\x00\x02\x00",
];
pub static FREESIA_MATURE_WILTED: Sprite = Sprite {
    width: 13,
    height: 17,
    frames: FREESIA_MATURE_WILTED_FRAMES,
    fill_frames: None,
};

const FREESIA_MATURE_DEAD_FRAMES: &[&[u8]] = &[
    b"\x06\x00\x09\x00\x53\x80\xb1\x80\x11\x00\x74\x00\x9a\x00\x92\x00\x10\x00\x08\x00",
];
pub static FREESIA_MATURE_DEAD: Sprite = Sprite {
    width: 9,
    height: 10,
    frames: FREESIA_MATURE_DEAD_FRAMES,
    fill_frames: None,
};

const FREESIA_THRIVING_FRAMES: &[&[u8]] = &[
    b"\x01\x00\x0a\xa0\x05\x40\x07\xc0\x07\xc0\x03\x80\x01\x00\x01\x00\x01\x10\x71\x30\x39\x70\x1d\x60\x0d\xc6\xc7\x9c\x73\xb8\x3d\x70\x1f\xe0\x0f\xc0\x07\x80\x01\x00\x01\x00\x01\x00",
];
pub static FREESIA_THRIVING: Sprite = Sprite {
    width: 15,
    height: 22,
    frames: FREESIA_THRIVING_FRAMES,
    fill_frames: None,
};

const FREESIA_THRIVING_WILTED_FRAMES: &[&[u8]] = &[
    b"\x00\x00\x01\x40\x03\x90\x01\xe8\x01\xf0\x01\xe8\x02\x10\x02\x00\x02\x00\x02\x00\x02\x70\x03\xf8\x7b\x00\xff\x20\xc2\x70\x83\xd0\x3f\x90\x77\x00\xe2\x00\x42\x00\x02\x00",
];
pub static FREESIA_THRIVING_WILTED: Sprite = Sprite {
    width: 13,
    height: 21,
    frames: FREESIA_THRIVING_WILTED_FRAMES,
    fill_frames: None,
};

const FREESIA_THRIVING_DEAD_FRAMES: &[&[u8]] = &[
    b"\x03\x80\x04\xc0\x09\xc0\x10\xe0\x3c\xe0\x52\x40\x10\xa0\x54\x00\xba\x00\x90\x00\x08\x00\x04\x00",
];
pub static FREESIA_THRIVING_DEAD: Sprite = Sprite {
    width: 11,
    height: 12,
    frames: FREESIA_THRIVING_DEAD_FRAMES,
    fill_frames: None,
};

const TINY_FLOWER_FRAMES: &[&[u8]] = &[
    b"\x60\x90\x60",
];
pub static TINY_FLOWER: Sprite = Sprite {
    width: 4,
    height: 3,
    frames: TINY_FLOWER_FRAMES,
    fill_frames: None,
};
