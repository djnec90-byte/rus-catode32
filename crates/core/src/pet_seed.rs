//! Pet seed to personality / favorites derivation.

use crate::t;

use crate::{
    context::{FoodItem, ToyVariant},
    rand::xorshift32,
    scene::SceneId,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PetGender {
    Tom,
    Queen,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum StarSign {
    Aries = 0,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl StarSign {
    pub fn from_index(idx: u32) -> Self {
        Self::ALL[(idx as usize) % Self::ALL.len()]
    }

    /// Stable English identifier used as the save-file serialization key.
    /// Do not translate — saves would break across language changes.
    pub fn name(self) -> &'static str {
        match self {
            StarSign::Aries => "Aries",
            StarSign::Taurus => "Taurus",
            StarSign::Gemini => "Gemini",
            StarSign::Cancer => "Cancer",
            StarSign::Leo => "Leo",
            StarSign::Virgo => "Virgo",
            StarSign::Libra => "Libra",
            StarSign::Scorpio => "Scorpio",
            StarSign::Sagittarius => "Sagittarius",
            StarSign::Capricorn => "Capricorn",
            StarSign::Aquarius => "Aquarius",
            StarSign::Pisces => "Pisces",
        }
    }

    /// Translated display label for UI rendering.
    pub fn label(self) -> &'static str {
        match self {
            StarSign::Aries => t!("Aries"),
            StarSign::Taurus => t!("Taurus"),
            StarSign::Gemini => t!("Gemini"),
            StarSign::Cancer => t!("Cancer"),
            StarSign::Leo => t!("Leo"),
            StarSign::Virgo => t!("Virgo"),
            StarSign::Libra => t!("Libra"),
            StarSign::Scorpio => t!("Scorpio"),
            StarSign::Sagittarius => t!("Sagittarius"),
            StarSign::Capricorn => t!("Capricorn"),
            StarSign::Aquarius => t!("Aquarius"),
            StarSign::Pisces => t!("Pisces"),
        }
    }

    /// Legacy ASCII lowercase used in a few places that still need a
    /// `&'static str`. Prefer `to_lower(s.label())` for new code so the
    /// embedded form respects the active translation.
    pub fn lower_name(self) -> &'static str {
        match self {
            StarSign::Aries => "aries",
            StarSign::Taurus => "taurus",
            StarSign::Gemini => "gemini",
            StarSign::Cancer => "cancer",
            StarSign::Leo => "leo",
            StarSign::Virgo => "virgo",
            StarSign::Libra => "libra",
            StarSign::Scorpio => "scorpio",
            StarSign::Sagittarius => "sagittarius",
            StarSign::Capricorn => "capricorn",
            StarSign::Aquarius => "aquarius",
            StarSign::Pisces => "pisces",
        }
    }

    pub const ALL: [StarSign; 12] = [
        StarSign::Aries,
        StarSign::Taurus,
        StarSign::Gemini,
        StarSign::Cancer,
        StarSign::Leo,
        StarSign::Virgo,
        StarSign::Libra,
        StarSign::Scorpio,
        StarSign::Sagittarius,
        StarSign::Capricorn,
        StarSign::Aquarius,
        StarSign::Pisces,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Temperament {
    Bold = 0,
    Loyal,
    Mischievous,
    Curious,
    Sociable,
}

impl Temperament {
    pub fn from_dominant_index(idx: usize) -> Self {
        match idx {
            0 => Temperament::Bold,
            1 => Temperament::Loyal,
            2 => Temperament::Mischievous,
            3 => Temperament::Curious,
            _ => Temperament::Sociable,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Temperament::Bold => t!("Bold"),
            Temperament::Loyal => t!("Loyal"),
            Temperament::Mischievous => t!("Mischievous"),
            Temperament::Curious => t!("Curious"),
            Temperament::Sociable => t!("Sociable"),
        }
    }

    pub fn lower_label(self) -> &'static str {
        match self {
            Temperament::Bold => "bold",
            Temperament::Loyal => "loyal",
            Temperament::Mischievous => "mischievous",
            Temperament::Curious => "curious",
            Temperament::Sociable => "sociable",
        }
    }
}

pub fn fav_weather_from_index(idx: u32) -> crate::context::FavWeather {
    use crate::context::FavWeather;
    match idx % 4 {
        0 => FavWeather::Sunny,
        1 => FavWeather::Rainy,
        2 => FavWeather::Snowy,
        _ => FavWeather::Overcast,
    }
}

const MEALS: [FoodItem; 13] = [
    FoodItem::Kibble,
    FoodItem::Cod,
    FoodItem::Haddock,
    FoodItem::Trout,
    FoodItem::Shrimp,
    FoodItem::Herring,
    FoodItem::Turkey,
    FoodItem::Tuna,
    FoodItem::Salmon,
    FoodItem::Chicken,
    FoodItem::Liver,
    FoodItem::Beef,
    FoodItem::Lamb,
];

const SNACKS: [FoodItem; 9] = [
    FoodItem::Carrots,
    FoodItem::Pumpkin,
    FoodItem::Treats,
    FoodItem::FishBite,
    FoodItem::Eggs,
    FoodItem::Nugget,
    FoodItem::Milk,
    FoodItem::ChewStick,
    FoodItem::Puree,
];

const TOY_VARIANTS: [ToyVariant; 5] = [
    ToyVariant::String_,
    ToyVariant::Feather,
    ToyVariant::Ball,
    ToyVariant::Laser,
    ToyVariant::Mouse,
];

const LOCATIONS: [SceneId; 4] = [
    SceneId::Outside,
    SceneId::Kitchen,
    SceneId::Treehouse,
    SceneId::Bedroom,
];

const TRAIT_MAGNITUDE: u32 = 10;
const PERSONALITY_TRAITS: usize = 5;

/// Returns five `(courage, loyalty, mischievousness, curiosity, sociability)`
/// offsets in roughly `[-TRAIT_MAGNITUDE, +TRAIT_MAGNITUDE]`, mean-centered.
pub fn derive_trait_offsets(seed: u64) -> [i32; PERSONALITY_TRAITS] {
    let mut state = ((seed ^ (seed >> 32)) & 0xFFFF_FFFF) as u32;
    if state == 0 {
        state = 1;
    }
    let span = 2 * TRAIT_MAGNITUDE + 1;
    let mut raw = [0u32; PERSONALITY_TRAITS];
    for slot in raw.iter_mut() {
        *slot = xorshift32(&mut state) % span;
    }
    let mean: u32 = raw.iter().sum::<u32>() / PERSONALITY_TRAITS as u32;
    let mut out = [0i32; PERSONALITY_TRAITS];
    for (i, &v) in raw.iter().enumerate() {
        out[i] = v as i32 - mean as i32;
    }
    out
}

#[derive(Clone, Copy)]
pub struct DerivedFavorites {
    pub pet_gender: PetGender,
    pub fav_weather: crate::context::FavWeather,
    pub star_sign: StarSign,
    pub fav_meal: FoodItem,
    pub least_fav_meal: FoodItem,
    pub fav_snack: FoodItem,
    pub least_fav_snack: FoodItem,
    pub fav_toy: ToyVariant,
    pub least_fav_toy: ToyVariant,
    pub fav_location: SceneId,
    pub least_fav_location: SceneId,
}

/// Uses the upper 32 bits of `seed` as the xorshift starting state, keeping
/// derivation independent from `derive_trait_offsets` (which uses the lower).
pub fn derive_favorites(seed: u64) -> DerivedFavorites {
    let mut state = ((seed >> 32) & 0xFFFF_FFFF) as u32;
    if state == 0 {
        state = 1;
    }
    let mut next = || xorshift32(&mut state);

    let gender = if next() % 2 == 0 {
        PetGender::Tom
    } else {
        PetGender::Queen
    };
    let fav_weather = fav_weather_from_index(next());

    fn pick_two<T: Copy, const N: usize>(items: &[T; N], next: &mut impl FnMut() -> u32) -> (T, T) {
        let n = N as u32;
        let fi = next() % n;
        let li = (fi + 1 + next() % (n - 1)) % n;
        (items[fi as usize], items[li as usize])
    }

    let star_sign = StarSign::from_index(next());
    let (fav_meal, least_fav_meal) = pick_two(&MEALS, &mut next);
    let (fav_snack, least_fav_snack) = pick_two(&SNACKS, &mut next);
    let (fav_toy, least_fav_toy) = pick_two(&TOY_VARIANTS, &mut next);
    let (fav_location, least_fav_location) = pick_two(&LOCATIONS, &mut next);

    DerivedFavorites {
        pet_gender: gender,
        fav_weather,
        star_sign,
        fav_meal,
        least_fav_meal,
        fav_snack,
        least_fav_snack,
        fav_toy,
        least_fav_toy,
        fav_location,
        least_fav_location,
    }
}
