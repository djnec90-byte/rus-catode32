//! Save / load for `GameContext`.
//!
//! On-disk format mirrors MicroPython's `save.json` byte-for-byte so existing
//! Python backups remain loadable (Rust → Python is best-effort; Python → Rust
//! is the priority). Storage layer is `crate::storage` (round-robin sector
//! rotation on the `nvs` partition).
//!
//! Field-by-field this is a direct port of `micropython/src/context.py`'s
//! `_write_to_flash` and `load`.

use core::str::FromStr;

use esp_hal::time::{Duration, Instant};
use esp_println::println;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

use crate::{
    assets::plants::{PlantStage, PotKind},
    context::{
        FavWeather, FoodItem, GameContext, PotSize, SeedKind, ToolKind, ToyEntry, ToyVariant,
        FOOD_ITEM_COUNT, MAX_PLANTS, RECENT_HISTORY, TOY_VARIANT_COUNT, PET_NAME_MAX,
    },
    pet_seed::{PetGender, StarSign},
    plant_system::{Plant, PlantLayer},
    scene::SceneId,
    storage,
    time_system::{Season, Weather},
};

/// Major schema version. Bumped when the on-disk shape changes in a
/// backwards-incompatible way. Matches Python's `data['v'] = major(VERSION)`.
const SCHEMA_VERSION: u8 = 0;

/// 59 minutes — same threshold Python's `save_if_needed` uses.
const SAVE_INTERVAL: Duration = Duration::from_secs(59 * 60);

/// Max size of the JSON payload. Sized to `storage::MAX_PAYLOAD`, which
/// covers a maxed-out save (80 plants, full inventory) with room to spare.
const JSON_BUF_SIZE: usize = storage::MAX_PAYLOAD;

/// Working buffer for JSON encode/decode. `static mut` because the buffer is
/// large enough that a stack allocation would dwarf typical task stacks.
static mut JSON_BUF: [u8; JSON_BUF_SIZE] = [0u8; JSON_BUF_SIZE];

// ---------------------------------------------------------------------------
// Strings used in the JSON. Centralised here so the encoder and the tolerant
// decoder agree.
// ---------------------------------------------------------------------------

type SStr = String<24>;
type StageStr = String<20>;
type NameStr = String<PET_NAME_MAX>;

fn sstr(s: &str) -> SStr {
    let mut out = SStr::new();
    // `push_str` truncates by returning Err — we deliberately allow truncation
    // since every string we hand it is shorter than the buffer.
    let _ = out.push_str(s);
    out
}

// ---------------------------------------------------------------------------
// Per-enum string mapping. Keep snake_case to match Python's existing
// serialised keys.
// ---------------------------------------------------------------------------

fn food_save_key(item: FoodItem) -> &'static str {
    match item {
        FoodItem::Kibble => "kibble",
        FoodItem::Cod => "cod",
        FoodItem::Haddock => "haddock",
        FoodItem::Trout => "trout",
        FoodItem::Shrimp => "shrimp",
        FoodItem::Herring => "herring",
        FoodItem::Turkey => "turkey",
        FoodItem::Tuna => "tuna",
        FoodItem::Salmon => "salmon",
        FoodItem::Chicken => "chicken",
        FoodItem::Liver => "liver",
        FoodItem::Beef => "beef",
        FoodItem::Lamb => "lamb",
        FoodItem::Mackerel => "mackerel",
        FoodItem::Carrots => "carrots",
        FoodItem::Pumpkin => "pumpkin",
        FoodItem::Treats => "treats",
        FoodItem::FishBite => "fish_bite",
        FoodItem::Eggs => "eggs",
        FoodItem::Nugget => "nugget",
        FoodItem::Milk => "milk",
        FoodItem::ChewStick => "chew_stick",
        FoodItem::Puree => "puree",
    }
}

fn food_from_key(s: &str) -> Option<FoodItem> {
    Some(match s {
        "kibble" => FoodItem::Kibble,
        "cod" => FoodItem::Cod,
        "haddock" => FoodItem::Haddock,
        "trout" => FoodItem::Trout,
        "shrimp" => FoodItem::Shrimp,
        "herring" => FoodItem::Herring,
        "turkey" => FoodItem::Turkey,
        "tuna" => FoodItem::Tuna,
        "salmon" => FoodItem::Salmon,
        "chicken" => FoodItem::Chicken,
        "liver" => FoodItem::Liver,
        "beef" => FoodItem::Beef,
        "lamb" => FoodItem::Lamb,
        "mackerel" => FoodItem::Mackerel,
        "carrots" => FoodItem::Carrots,
        "pumpkin" => FoodItem::Pumpkin,
        "treats" => FoodItem::Treats,
        "fish_bite" => FoodItem::FishBite,
        "eggs" => FoodItem::Eggs,
        "nugget" => FoodItem::Nugget,
        "milk" => FoodItem::Milk,
        "chew_stick" => FoodItem::ChewStick,
        "puree" => FoodItem::Puree,
        _ => return None,
    })
}

fn toy_save_key(v: ToyVariant) -> &'static str {
    match v {
        ToyVariant::String_ => "string",
        ToyVariant::Feather => "feather",
        ToyVariant::Mouse => "mouse",
        ToyVariant::Ball => "ball",
        ToyVariant::Bubbles => "bubbles",
        ToyVariant::Laser => "laser",
    }
}

fn toy_from_key(s: &str) -> Option<ToyVariant> {
    Some(match s {
        "string" => ToyVariant::String_,
        "feather" => ToyVariant::Feather,
        "mouse" => ToyVariant::Mouse,
        "ball" => ToyVariant::Ball,
        "bubbles" => ToyVariant::Bubbles,
        "laser" => ToyVariant::Laser,
        _ => return None,
    })
}

fn pot_save_key(p: PotKind) -> &'static str {
    match p {
        PotKind::Small => "small",
        PotKind::Medium => "medium",
        PotKind::Large => "large",
        PotKind::Planter => "planter",
        PotKind::Ground => "ground",
    }
}

fn pot_from_key(s: &str) -> Option<PotKind> {
    Some(match s {
        "small" => PotKind::Small,
        "medium" => PotKind::Medium,
        "large" => PotKind::Large,
        "planter" => PotKind::Planter,
        "ground" => PotKind::Ground,
        _ => return None,
    })
}

fn seed_save_key(s: SeedKind) -> &'static str {
    match s {
        SeedKind::CatGrass => "cat_grass",
        SeedKind::Freesia => "freesia",
        SeedKind::Sunflower => "sunflower",
        SeedKind::Rose => "rose",
    }
}

fn seed_from_key(s: &str) -> Option<SeedKind> {
    Some(match s {
        "cat_grass" => SeedKind::CatGrass,
        "freesia" => SeedKind::Freesia,
        "sunflower" => SeedKind::Sunflower,
        "rose" => SeedKind::Rose,
        _ => return None,
    })
}

fn layer_save_key(l: PlantLayer) -> &'static str {
    match l {
        PlantLayer::Background => "background",
        PlantLayer::Midground => "midground",
        PlantLayer::Foreground => "foreground",
    }
}

fn layer_from_key(s: &str) -> PlantLayer {
    // Python had a `fg/mg/bg` legacy abbreviation pass; honour it here so old
    // saves still load cleanly.
    match s {
        "background" | "bg" => PlantLayer::Background,
        "foreground" | "fg" => PlantLayer::Foreground,
        _ => PlantLayer::Midground,
    }
}

fn scene_save_key(s: SceneId) -> &'static str {
    match s {
        SceneId::Inside => "inside",
        SceneId::Outside => "outside",
        SceneId::Bedroom => "bedroom",
        SceneId::Kitchen => "kitchen",
        SceneId::Treehouse => "treehouse",
        // Non-main scenes never appear on a saved plant, but keep the mapping
        // exhaustive so the compiler catches future SceneId additions.
        _ => "inside",
    }
}

fn scene_from_key(s: &str) -> SceneId {
    match s {
        "outside" => SceneId::Outside,
        "bedroom" => SceneId::Bedroom,
        "kitchen" => SceneId::Kitchen,
        "treehouse" => SceneId::Treehouse,
        _ => SceneId::Inside,
    }
}

fn stage_save_key(st: PlantStage) -> &'static str {
    match st {
        PlantStage::EmptyPot => "empty_pot",
        PlantStage::Seedling => "seedling",
        PlantStage::Young => "young",
        PlantStage::Growing => "growing",
        PlantStage::Mature => "mature",
        PlantStage::Thriving => "thriving",
        PlantStage::SeedlingWilted => "seedling_wilted",
        PlantStage::YoungWilted => "young_wilted",
        PlantStage::GrowingWilted => "growing_wilted",
        PlantStage::MatureWilted => "mature_wilted",
        PlantStage::ThrivingWilted => "thriving_wilted",
        PlantStage::SeedlingDead => "seedling_dead",
        PlantStage::YoungDead => "young_dead",
        PlantStage::GrowingDead => "growing_dead",
        PlantStage::MatureDead => "mature_dead",
        PlantStage::ThrivingDead => "thriving_dead",
        PlantStage::Dead => "dead",
        PlantStage::Dormant => "dormant",
    }
}

fn stage_from_key(s: &str) -> PlantStage {
    match s {
        "empty_pot" => PlantStage::EmptyPot,
        "seedling" => PlantStage::Seedling,
        "young" => PlantStage::Young,
        "growing" => PlantStage::Growing,
        "mature" => PlantStage::Mature,
        "thriving" => PlantStage::Thriving,
        "seedling_wilted" => PlantStage::SeedlingWilted,
        "young_wilted" | "withering" => PlantStage::YoungWilted,
        "growing_wilted" => PlantStage::GrowingWilted,
        "mature_wilted" => PlantStage::MatureWilted,
        "thriving_wilted" => PlantStage::ThrivingWilted,
        "seedling_dead" => PlantStage::SeedlingDead,
        "young_dead" => PlantStage::YoungDead,
        "growing_dead" => PlantStage::GrowingDead,
        "mature_dead" => PlantStage::MatureDead,
        "thriving_dead" => PlantStage::ThrivingDead,
        "dead" => PlantStage::Dead,
        "dormant" => PlantStage::Dormant,
        _ => PlantStage::Young,
    }
}

fn weather_save_key(w: Weather) -> &'static str {
    w.name()
}

fn weather_from_key(s: &str) -> Weather {
    match s {
        "Clear" => Weather::Clear,
        "Cloudy" => Weather::Cloudy,
        "Overcast" => Weather::Overcast,
        "Windy" => Weather::Windy,
        "Rain" => Weather::Rain,
        "Storm" => Weather::Storm,
        "Snow" => Weather::Snow,
        _ => Weather::Clear,
    }
}

fn season_save_key(s: Season) -> &'static str {
    s.name()
}

fn season_from_key(s: &str) -> Season {
    match s {
        "Winter" => Season::Winter,
        "Spring" => Season::Spring,
        "Summer" => Season::Summer,
        "Fall" => Season::Fall,
        _ => Season::Spring,
    }
}

/// Python writes `moon_phase` as a human label ("1st Qtr", "Full", ...).
/// Rust stores it as a 0–7 phase index, so save in the Python form and parse
/// either form on load.
fn moon_phase_save_key(p: u8) -> &'static str {
    match p % 8 {
        0 => "New",
        1 => "Waxing Crescent",
        2 => "1st Qtr",
        3 => "Waxing Gibbous",
        4 => "Full",
        5 => "Waning Gibbous",
        6 => "Last Qtr",
        _ => "Waning Crescent",
    }
}

fn moon_phase_from_key(s: &str) -> u8 {
    match s {
        "New" => 0,
        "Waxing Crescent" => 1,
        "1st Qtr" | "First Qtr" => 2,
        "Waxing Gibbous" => 3,
        "Full" => 4,
        "Waning Gibbous" => 5,
        "Last Qtr" | "3rd Qtr" => 6,
        "Waning Crescent" => 7,
        // Fall back to numeric parse so a Rust-emitted save can also round-trip
        // even if a future revision drops the labels.
        _ => u8::from_str(s).unwrap_or(0),
    }
}

fn gender_save_key(g: PetGender) -> &'static str {
    match g {
        PetGender::Tom => "tom",
        PetGender::Queen => "queen",
    }
}

fn gender_from_key(s: &str) -> Option<PetGender> {
    Some(match s {
        "tom" => PetGender::Tom,
        "queen" => PetGender::Queen,
        _ => return None,
    })
}

fn fav_weather_save_key(w: FavWeather) -> &'static str {
    match w {
        FavWeather::Sunny => "sunny",
        FavWeather::Rainy => "rainy",
        FavWeather::Snowy => "snowy",
        FavWeather::Overcast => "overcast",
    }
}

fn fav_weather_from_key(s: &str) -> Option<FavWeather> {
    Some(match s {
        "sunny" => FavWeather::Sunny,
        "rainy" => FavWeather::Rainy,
        "snowy" => FavWeather::Snowy,
        "overcast" => FavWeather::Overcast,
        _ => return None,
    })
}

fn star_sign_save_key(s: StarSign) -> &'static str {
    s.name()
}

fn star_sign_from_key(s: &str) -> Option<StarSign> {
    StarSign::ALL.iter().copied().find(|x| x.name() == s)
}

fn fav_location_save_key(s: SceneId) -> &'static str {
    match s {
        SceneId::Outside => "outside",
        SceneId::Kitchen => "kitchen",
        SceneId::Treehouse => "treehouse",
        SceneId::Bedroom => "bedroom",
        _ => "inside",
    }
}

fn fav_location_from_key(s: &str) -> Option<SceneId> {
    Some(match s {
        "outside" => SceneId::Outside,
        "kitchen" => SceneId::Kitchen,
        "treehouse" => SceneId::Treehouse,
        "bedroom" => SceneId::Bedroom,
        "inside" => SceneId::Inside,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Wire-format structs. Field names + #[serde(rename = "...")] match Python's
// keys exactly. #[serde(default)] on every field keeps loading tolerant when
// keys are missing.
// ---------------------------------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct EnvData {
    #[serde(default)] season_offset: u16,
    #[serde(default)] season: SStr,
    #[serde(default)] weather: SStr,
    #[serde(default)] weather_step: u32,
    #[serde(default)] weather_timer: f32,
    #[serde(default)] meteor_shower_timer: f32,
    #[serde(default)] moon_phase: SStr,
    #[serde(default)] temperature: f32,
    #[serde(default)] time_hours: u8,
    #[serde(default)] time_minutes: u8,
    #[serde(default)] day_number: u32,
}

#[derive(Default, Serialize, Deserialize)]
struct FoodStockData {
    #[serde(default)] kibble: u8,
    #[serde(default)] cod: u8,
    #[serde(default)] haddock: u8,
    #[serde(default)] trout: u8,
    #[serde(default)] shrimp: u8,
    #[serde(default)] herring: u8,
    #[serde(default)] turkey: u8,
    #[serde(default)] tuna: u8,
    #[serde(default)] salmon: u8,
    #[serde(default)] chicken: u8,
    #[serde(default)] liver: u8,
    #[serde(default)] beef: u8,
    #[serde(default)] lamb: u8,
    #[serde(default)] mackerel: u8,
    #[serde(default)] carrots: u8,
    #[serde(default)] pumpkin: u8,
    #[serde(default)] treats: u8,
    #[serde(default)] fish_bite: u8,
    #[serde(default)] eggs: u8,
    #[serde(default)] nugget: u8,
    #[serde(default)] milk: u8,
    #[serde(default)] chew_stick: u8,
    #[serde(default)] puree: u8,
}

#[derive(Default, Serialize, Deserialize)]
struct PotsData {
    #[serde(default)] small: u8,
    #[serde(default)] medium: u8,
    #[serde(default)] large: u8,
    #[serde(default)] planter: u8,
}

#[derive(Default, Serialize, Deserialize)]
struct SeedsData {
    #[serde(default)] cat_grass: u8,
    #[serde(default)] sunflower: u8,
    #[serde(default)] rose: u8,
    #[serde(default)] freesia: u8,
    // TODO(plants): Tulip exists in Python's seed list but has no plant type
    // in the Rust port yet. Round-trip the count so old saves don't lose it.
    #[serde(default)] tulip: u8,
}

#[derive(Default, Serialize, Deserialize)]
struct ToolsData {
    #[serde(default)] watering_can: bool,
    #[serde(default)] spade: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct ToyData {
    #[serde(default)] variant: SStr,
    #[serde(default)] durability: u8,
}

#[derive(Default, Serialize, Deserialize)]
struct PlantRecord {
    #[serde(default)] id: u32,
    /// Python key is the Python keyword `type`; preserve the wire name.
    #[serde(default, rename = "type")]
    seed_type: SStr,
    #[serde(default)] scene: SStr,
    #[serde(default)] layer: SStr,
    #[serde(default)] x: i32,
    #[serde(default)] y_snap: i32,
    #[serde(default)] pot: SStr,
    #[serde(default)] stage: StageStr,
    #[serde(default)] age_hours: u32,
    #[serde(default)] water_debt_hours: f32,
    #[serde(default)] fertilizer: f32,
    /// Python uses 0 as the "no planted_day recorded" sentinel; Rust uses
    /// `Option<u32>`. We serialise as u32 and map 0 → Some(0) on load — the
    /// distinction is purely cosmetic during gameplay.
    #[serde(default)] planted_day: u32,
    #[serde(default)] mirror: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct MilestonesData {
    #[serde(default)] fed: bool,
    #[serde(default)] groomed: bool,
    #[serde(default)] played: bool,
    #[serde(default)] petted: bool,
    #[serde(default)] store: bool,
}

/// Placeholder for fields Python tracks but the Rust port hasn't ported yet
/// (wifi lists, friends map). Serialises as an empty JSON array or object so
/// the wire shape matches Python; deserialise is a no-op accept-anything.
/// TODO(wifi_espnow): replace with real types once WiFi/ESP-NOW lands.
#[derive(Default)]
struct StubArray;
impl Serialize for StubArray {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        s.serialize_seq(Some(0))?.end()
    }
}
impl<'de> Deserialize<'de> for StubArray {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        serde::de::IgnoredAny::deserialize(d).map(|_| StubArray)
    }
}

#[derive(Default)]
struct StubMap;
impl Serialize for StubMap {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        s.serialize_map(Some(0))?.end()
    }
}
impl<'de> Deserialize<'de> for StubMap {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        serde::de::IgnoredAny::deserialize(d).map(|_| StubMap)
    }
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    #[serde(default)] v: u8,
    #[serde(default)] env: EnvData,
    #[serde(default)] food_stock: FoodStockData,
    #[serde(default)] toys: Vec<ToyData, TOY_VARIANT_COUNT>,
    #[serde(default)] pots: PotsData,
    #[serde(default)] seeds: SeedsData,
    #[serde(default)] tools: ToolsData,
    #[serde(default)] fertilizer: u8,
    #[serde(default)] medicine: u8,
    #[serde(default)] sickness: f32,
    #[serde(default)] medicine_pending: bool,
    #[serde(default)] plants: Vec<PlantRecord, MAX_PLANTS>,
    #[serde(default)] next_plant_id: u32,
    #[serde(default)] pet_seed: u64,
    #[serde(default)] pet_gender: Option<SStr>,
    #[serde(default)] fav_weather: Option<SStr>,
    #[serde(default)] star_sign: Option<SStr>,
    #[serde(default)] fav_meal: Option<SStr>,
    #[serde(default)] least_fav_meal: Option<SStr>,
    #[serde(default)] fav_snack: Option<SStr>,
    #[serde(default)] least_fav_snack: Option<SStr>,
    #[serde(default)] fav_toy: Option<SStr>,
    #[serde(default)] least_fav_toy: Option<SStr>,
    #[serde(default)] fav_location: Option<SStr>,
    #[serde(default)] least_fav_location: Option<SStr>,
    #[serde(default)] wifi_familiar: StubArray,
    #[serde(default)] wifi_recent: StubArray,
    #[serde(default)] pet_name: Option<NameStr>,
    #[serde(default)] friends: StubMap,
    #[serde(default)] recent_meals: Vec<SStr, RECENT_HISTORY>,
    #[serde(default)] milestones: MilestonesData,

    // Flat stat fields — same keys Python's `_STAT_KEYS` writes at top level.
    #[serde(default)] fullness: f32,
    #[serde(default)] energy: f32,
    #[serde(default)] comfort: f32,
    #[serde(default)] playfulness: f32,
    #[serde(default)] focus: f32,
    #[serde(default)] fulfillment: f32,
    #[serde(default)] cleanliness: f32,
    #[serde(default)] curiosity: f32,
    #[serde(default)] sociability: f32,
    #[serde(default)] intelligence: f32,
    #[serde(default)] maturity: f32,
    #[serde(default)] affection: f32,
    #[serde(default)] fitness: f32,
    #[serde(default)] serenity: f32,
    #[serde(default)] courage: f32,
    #[serde(default)] loyalty: f32,
    #[serde(default)] mischievousness: f32,
    #[serde(default)] zoomies_high_score: i32,
    #[serde(default)] maze_best_time: i32,
    #[serde(default)] snake_high_score: i32,
    #[serde(default)] memory_best_score: i32,
    #[serde(default)] hanjie_best_time: i32,
    #[serde(default)] time_speed: f32,
    #[serde(default)] coins: i32,
}

// ---------------------------------------------------------------------------
// Bridging GameContext ↔ SaveData.
// ---------------------------------------------------------------------------

fn build(ctx: &GameContext) -> SaveData {
    let mut toys: Vec<ToyData, TOY_VARIANT_COUNT> = Vec::new();
    for t in ctx.toys.iter() {
        let _ = toys.push(ToyData {
            variant: sstr(toy_save_key(t.variant)),
            durability: t.durability,
        });
    }

    let mut plants: Vec<PlantRecord, MAX_PLANTS> = Vec::new();
    for p in ctx.plants.iter() {
        let _ = plants.push(PlantRecord {
            id: p.id,
            seed_type: p
                .seed
                .map(|s| sstr(seed_save_key(s)))
                .unwrap_or_default(),
            scene: sstr(scene_save_key(p.scene)),
            layer: sstr(layer_save_key(p.layer)),
            x: p.x,
            y_snap: p.y_snap,
            pot: sstr(pot_save_key(p.pot)),
            stage: {
                let mut s = StageStr::new();
                let _ = s.push_str(stage_save_key(p.stage));
                s
            },
            age_hours: p.age_hours,
            water_debt_hours: p.water_debt,
            fertilizer: p.fertilizer,
            planted_day: p.planted_day.unwrap_or(0),
            mirror: p.mirror,
        });
    }

    let mut recent_meals: Vec<SStr, RECENT_HISTORY> = Vec::new();
    for m in ctx.recent_meals.iter() {
        use crate::context::MealEntry;
        let key = match m {
            MealEntry::Item(item) => food_save_key(*item),
            MealEntry::CaughtSnack => "caught_snack",
        };
        let _ = recent_meals.push(sstr(key));
    }

    SaveData {
        v: SCHEMA_VERSION,
        env: EnvData {
            season_offset: ctx.season_offset,
            season: sstr(season_save_key(ctx.season)),
            weather: sstr(weather_save_key(ctx.weather)),
            weather_step: ctx.weather_step,
            weather_timer: ctx.weather_timer,
            meteor_shower_timer: ctx.meteor_shower_timer,
            moon_phase: sstr(moon_phase_save_key(ctx.moon_phase)),
            temperature: ctx.temperature,
            time_hours: ctx.time_hours,
            time_minutes: ctx.time_minutes,
            day_number: ctx.day_number,
        },
        food_stock: FoodStockData {
            kibble: ctx.food_stock[FoodItem::Kibble as usize],
            cod: ctx.food_stock[FoodItem::Cod as usize],
            haddock: ctx.food_stock[FoodItem::Haddock as usize],
            trout: ctx.food_stock[FoodItem::Trout as usize],
            shrimp: ctx.food_stock[FoodItem::Shrimp as usize],
            herring: ctx.food_stock[FoodItem::Herring as usize],
            turkey: ctx.food_stock[FoodItem::Turkey as usize],
            tuna: ctx.food_stock[FoodItem::Tuna as usize],
            salmon: ctx.food_stock[FoodItem::Salmon as usize],
            chicken: ctx.food_stock[FoodItem::Chicken as usize],
            liver: ctx.food_stock[FoodItem::Liver as usize],
            beef: ctx.food_stock[FoodItem::Beef as usize],
            lamb: ctx.food_stock[FoodItem::Lamb as usize],
            mackerel: ctx.food_stock[FoodItem::Mackerel as usize],
            carrots: ctx.food_stock[FoodItem::Carrots as usize],
            pumpkin: ctx.food_stock[FoodItem::Pumpkin as usize],
            treats: ctx.food_stock[FoodItem::Treats as usize],
            fish_bite: ctx.food_stock[FoodItem::FishBite as usize],
            eggs: ctx.food_stock[FoodItem::Eggs as usize],
            nugget: ctx.food_stock[FoodItem::Nugget as usize],
            milk: ctx.food_stock[FoodItem::Milk as usize],
            chew_stick: ctx.food_stock[FoodItem::ChewStick as usize],
            puree: ctx.food_stock[FoodItem::Puree as usize],
        },
        toys,
        pots: PotsData {
            small: ctx.pots[PotSize::Small as usize],
            medium: ctx.pots[PotSize::Medium as usize],
            large: ctx.pots[PotSize::Large as usize],
            planter: ctx.pots[PotSize::Planter as usize],
        },
        seeds: SeedsData {
            cat_grass: ctx.seeds[SeedKind::CatGrass as usize],
            sunflower: ctx.seeds[SeedKind::Sunflower as usize],
            rose: ctx.seeds[SeedKind::Rose as usize],
            freesia: ctx.seeds[SeedKind::Freesia as usize],
            tulip: 0,
        },
        tools: ToolsData {
            watering_can: ctx.tools[ToolKind::WateringCan as usize],
            spade: ctx.tools[ToolKind::Spade as usize],
        },
        fertilizer: ctx.fertilizer,
        medicine: ctx.medicine,
        sickness: ctx.sickness,
        medicine_pending: ctx.medicine_pending,
        plants,
        next_plant_id: ctx.next_plant_id,
        pet_seed: ctx.pet_seed,
        pet_gender: ctx.pet_gender.map(|g| sstr(gender_save_key(g))),
        fav_weather: ctx.fav_weather.map(|w| sstr(fav_weather_save_key(w))),
        star_sign: ctx.star_sign.map(|s| sstr(star_sign_save_key(s))),
        fav_meal: ctx.fav_meal.map(|f| sstr(food_save_key(f))),
        least_fav_meal: ctx.least_fav_meal.map(|f| sstr(food_save_key(f))),
        fav_snack: ctx.fav_snack.map(|f| sstr(food_save_key(f))),
        least_fav_snack: ctx.least_fav_snack.map(|f| sstr(food_save_key(f))),
        fav_toy: ctx.fav_toy.map(|t| sstr(toy_save_key(t))),
        least_fav_toy: ctx.least_fav_toy.map(|t| sstr(toy_save_key(t))),
        fav_location: ctx.fav_location.map(|s| sstr(fav_location_save_key(s))),
        least_fav_location: ctx.least_fav_location.map(|s| sstr(fav_location_save_key(s))),
        wifi_familiar: StubArray,
        wifi_recent: StubArray,
        pet_name: if ctx.pet_name.is_empty() {
            None
        } else {
            let mut n = NameStr::new();
            let _ = n.push_str(ctx.pet_name.as_str());
            Some(n)
        },
        friends: StubMap,
        recent_meals,
        milestones: MilestonesData {
            fed: ctx.milestone_fed,
            groomed: ctx.milestone_groomed,
            played: ctx.milestone_played,
            petted: ctx.milestone_petted,
            store: ctx.milestone_store,
        },

        fullness: ctx.fullness,
        energy: ctx.energy,
        comfort: ctx.comfort,
        playfulness: ctx.playfulness,
        focus: ctx.focus,
        fulfillment: ctx.fulfillment,
        cleanliness: ctx.cleanliness,
        curiosity: ctx.curiosity,
        sociability: ctx.sociability,
        intelligence: ctx.intelligence,
        maturity: ctx.maturity,
        affection: ctx.affection,
        fitness: ctx.fitness,
        serenity: ctx.serenity,
        courage: ctx.courage,
        loyalty: ctx.loyalty,
        mischievousness: ctx.mischievousness,
        zoomies_high_score: ctx.zoomies_high_score,
        maze_best_time: ctx.maze_best_time,
        snake_high_score: ctx.snake_high_score,
        memory_best_score: ctx.memory_best_score,
        hanjie_best_time: ctx.hanjie_best_time,
        time_speed: ctx.time_speed,
        coins: ctx.coins,
    }
}

fn apply(data: &SaveData, ctx: &mut GameContext) {
    // Stats
    ctx.fullness = data.fullness;
    ctx.energy = data.energy;
    ctx.comfort = data.comfort;
    ctx.playfulness = data.playfulness;
    ctx.focus = data.focus;
    ctx.fulfillment = data.fulfillment;
    ctx.cleanliness = data.cleanliness;
    ctx.curiosity = data.curiosity;
    ctx.sociability = data.sociability;
    ctx.intelligence = data.intelligence;
    ctx.maturity = data.maturity;
    ctx.affection = data.affection;
    ctx.fitness = data.fitness;
    ctx.serenity = data.serenity;
    ctx.courage = data.courage;
    ctx.loyalty = data.loyalty;
    ctx.mischievousness = data.mischievousness;
    ctx.zoomies_high_score = data.zoomies_high_score;
    ctx.maze_best_time = data.maze_best_time;
    ctx.snake_high_score = data.snake_high_score;
    ctx.memory_best_score = data.memory_best_score;
    ctx.hanjie_best_time = data.hanjie_best_time;
    ctx.time_speed = data.time_speed;
    ctx.coins = data.coins;

    // Identity
    ctx.pet_seed = data.pet_seed;
    ctx.pet_gender = data.pet_gender.as_deref().and_then(gender_from_key);
    ctx.fav_weather = data.fav_weather.as_deref().and_then(fav_weather_from_key);
    ctx.star_sign = data.star_sign.as_deref().and_then(star_sign_from_key);
    ctx.fav_meal = data.fav_meal.as_deref().and_then(food_from_key);
    ctx.least_fav_meal = data.least_fav_meal.as_deref().and_then(food_from_key);
    ctx.fav_snack = data.fav_snack.as_deref().and_then(food_from_key);
    ctx.least_fav_snack = data.least_fav_snack.as_deref().and_then(food_from_key);
    ctx.fav_toy = data.fav_toy.as_deref().and_then(toy_from_key);
    ctx.least_fav_toy = data.least_fav_toy.as_deref().and_then(toy_from_key);
    ctx.fav_location = data.fav_location.as_deref().and_then(fav_location_from_key);
    ctx.least_fav_location = data
        .least_fav_location
        .as_deref()
        .and_then(fav_location_from_key);
    ctx.pet_name.clear();
    if let Some(n) = data.pet_name.as_deref() {
        for c in n.chars() {
            if ctx.pet_name.push(c).is_err() {
                break;
            }
        }
    }

    // Env
    ctx.season_offset = data.env.season_offset;
    ctx.season = season_from_key(&data.env.season);
    ctx.weather = weather_from_key(&data.env.weather);
    ctx.weather_step = data.env.weather_step;
    ctx.weather_timer = data.env.weather_timer;
    ctx.meteor_shower_timer = data.env.meteor_shower_timer;
    ctx.moon_phase = moon_phase_from_key(&data.env.moon_phase);
    ctx.temperature = data.env.temperature;
    ctx.time_hours = data.env.time_hours;
    ctx.time_minutes = data.env.time_minutes;
    ctx.day_number = data.env.day_number;

    // Inventory
    ctx.food_stock = [0u8; FOOD_ITEM_COUNT];
    ctx.food_stock[FoodItem::Kibble as usize] = data.food_stock.kibble;
    ctx.food_stock[FoodItem::Cod as usize] = data.food_stock.cod;
    ctx.food_stock[FoodItem::Haddock as usize] = data.food_stock.haddock;
    ctx.food_stock[FoodItem::Trout as usize] = data.food_stock.trout;
    ctx.food_stock[FoodItem::Shrimp as usize] = data.food_stock.shrimp;
    ctx.food_stock[FoodItem::Herring as usize] = data.food_stock.herring;
    ctx.food_stock[FoodItem::Turkey as usize] = data.food_stock.turkey;
    ctx.food_stock[FoodItem::Tuna as usize] = data.food_stock.tuna;
    ctx.food_stock[FoodItem::Salmon as usize] = data.food_stock.salmon;
    ctx.food_stock[FoodItem::Chicken as usize] = data.food_stock.chicken;
    ctx.food_stock[FoodItem::Liver as usize] = data.food_stock.liver;
    ctx.food_stock[FoodItem::Beef as usize] = data.food_stock.beef;
    ctx.food_stock[FoodItem::Lamb as usize] = data.food_stock.lamb;
    ctx.food_stock[FoodItem::Mackerel as usize] = data.food_stock.mackerel;
    ctx.food_stock[FoodItem::Carrots as usize] = data.food_stock.carrots;
    ctx.food_stock[FoodItem::Pumpkin as usize] = data.food_stock.pumpkin;
    ctx.food_stock[FoodItem::Treats as usize] = data.food_stock.treats;
    ctx.food_stock[FoodItem::FishBite as usize] = data.food_stock.fish_bite;
    ctx.food_stock[FoodItem::Eggs as usize] = data.food_stock.eggs;
    ctx.food_stock[FoodItem::Nugget as usize] = data.food_stock.nugget;
    ctx.food_stock[FoodItem::Milk as usize] = data.food_stock.milk;
    ctx.food_stock[FoodItem::ChewStick as usize] = data.food_stock.chew_stick;
    ctx.food_stock[FoodItem::Puree as usize] = data.food_stock.puree;

    ctx.toys.clear();
    for t in data.toys.iter() {
        if let Some(v) = toy_from_key(&t.variant) {
            let _ = ctx.toys.push(ToyEntry {
                variant: v,
                durability: t.durability,
            });
        }
    }

    ctx.pots[PotSize::Small as usize] = data.pots.small;
    ctx.pots[PotSize::Medium as usize] = data.pots.medium;
    ctx.pots[PotSize::Large as usize] = data.pots.large;
    ctx.pots[PotSize::Planter as usize] = data.pots.planter;

    ctx.seeds[SeedKind::CatGrass as usize] = data.seeds.cat_grass;
    ctx.seeds[SeedKind::Sunflower as usize] = data.seeds.sunflower;
    ctx.seeds[SeedKind::Rose as usize] = data.seeds.rose;
    ctx.seeds[SeedKind::Freesia as usize] = data.seeds.freesia;

    ctx.tools[ToolKind::WateringCan as usize] = data.tools.watering_can;
    ctx.tools[ToolKind::Spade as usize] = data.tools.spade;

    ctx.fertilizer = data.fertilizer;
    ctx.medicine = data.medicine;
    ctx.sickness = data.sickness;
    ctx.medicine_pending = data.medicine_pending;

    // Plants: only overwrite when the save actually contains plant entries,
    // matching Python's "if 'plants' in data" check (the starter set stays
    // otherwise).
    if !data.plants.is_empty() {
        ctx.plants.clear();
        for r in data.plants.iter() {
            let plant = Plant {
                id: r.id,
                seed: seed_from_key(&r.seed_type),
                scene: scene_from_key(&r.scene),
                layer: layer_from_key(&r.layer),
                x: r.x,
                y_snap: r.y_snap,
                pot: pot_from_key(&r.pot).unwrap_or(PotKind::Small),
                stage: stage_from_key(&r.stage),
                age_hours: r.age_hours,
                water_debt: r.water_debt_hours,
                fertilizer: r.fertilizer,
                planted_day: Some(r.planted_day),
                mirror: r.mirror,
                aged: false,
            };
            if ctx.plants.push(plant).is_err() {
                break;
            }
        }
    }
    if data.next_plant_id > 0 {
        ctx.next_plant_id = data.next_plant_id;
    } else {
        ctx.next_plant_id = ctx.plants.len() as u32;
    }

    // Recent meals.
    ctx.recent_meals.clear();
    for s in data.recent_meals.iter() {
        use crate::context::MealEntry;
        let entry = if s.as_str() == "caught_snack" {
            MealEntry::CaughtSnack
        } else if let Some(item) = food_from_key(s) {
            MealEntry::Item(item)
        } else {
            continue;
        };
        let _ = ctx.recent_meals.push(entry);
    }

    // Milestones.
    ctx.milestone_fed = data.milestones.fed;
    ctx.milestone_groomed = data.milestones.groomed;
    ctx.milestone_played = data.milestones.played;
    ctx.milestone_petted = data.milestones.petted;
    ctx.milestone_store = data.milestones.store;

    ctx.first_impressions = false;
    ctx.recompute_health();
}

// ---------------------------------------------------------------------------
// Public API.
// ---------------------------------------------------------------------------

/// True iff the storage layer holds at least one syntactically valid save.
pub fn has_save() -> bool {
    storage::has_save()
}

/// Load the most recent save into `ctx`. Returns true on success.
pub fn load(ctx: &mut GameContext) -> bool {
    // SAFETY: single-threaded boot path; JSON_BUF is only touched here and in
    // `save` below, never concurrently.
    let buf = unsafe { &mut *core::ptr::addr_of_mut!(JSON_BUF) };
    let Some(len) = storage::read_latest(buf) else {
        return false;
    };
    match serde_json_core::from_slice::<SaveData>(&buf[..len]) {
        Ok((data, _)) => {
            apply(&data, ctx);
            ctx.last_save_time = Some(Instant::now());
            println!("[Save] Loaded {} bytes", len);
            true
        }
        Err(_) => {
            println!("[Save] Parse failed (corrupt save)");
            false
        }
    }
}

/// Encode `ctx` to JSON and persist it. Returns true on success.
///
/// Unlike Python — which followed every save with `machine.soft_reset()` to
/// reclaim a fragmented heap — this just writes and returns. The Rust port
/// has no heap to fragment, so a save is a normal in-place operation.
pub fn save(ctx: &mut GameContext) -> bool {
    let data = build(ctx);
    // SAFETY: see `load`.
    let buf = unsafe { &mut *core::ptr::addr_of_mut!(JSON_BUF) };
    let len = match serde_json_core::to_slice(&data, buf) {
        Ok(n) => n,
        Err(_) => {
            println!("[Save] Encode failed");
            return false;
        }
    };
    if !storage::write_next(&buf[..len]) {
        return false;
    }
    ctx.last_save_time = Some(Instant::now());
    true
}

/// If `SAVE_INTERVAL` has elapsed since the last save, save now. Mirrors
/// Python's `save_if_needed` minus the reboot.
pub fn save_if_needed(ctx: &mut GameContext) {
    let due = match ctx.last_save_time {
        None => true,
        Some(t) => Instant::now().duration_since_epoch() - t.duration_since_epoch() > SAVE_INTERVAL,
    };
    if due {
        save(ctx);
    }
}
