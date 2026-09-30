//! Save / load for `GameContext`.
//!
//! Storage layer is `crate::storage` (round-robin sector rotation on the
//! `nvs` partition).

use core::str::FromStr;

use crate::platform::time::{Duration, Instant};
use crate::println;
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

use crate::{
    assets::plants::{PlantStage, PotKind},
    context::{
        FavWeather, FoodItem, GameContext, PotSize, SeedKind, ToolKind, ToyEntry, ToyVariant,
        WifiEntry, FOOD_ITEM_COUNT, MAX_PLANTS, PET_NAME_MAX, RECENT_HISTORY, TOY_VARIANT_COUNT,
        WIFI_FAMILIAR_MAX, WIFI_RECENT_MAX, WIFI_SSID_MAX,
    },
    pet_seed::{PetGender, StarSign},
    plant_system::{Plant, PlantLayer},
    scene::SceneId,
    storage,
    time_system::{Season, Weather},
    wifi_tracker,
};

const SCHEMA_VERSION: u8 = 0;
const SAVE_INTERVAL: Duration = Duration::from_secs(59 * 60);
const JSON_BUF_SIZE: usize = storage::MAX_PAYLOAD;

// Безопасный выровненный буфер, чтобы ESP32-C3 не зависал при чтении флеша
#[repr(align(4))]
struct AlignedJsonBuf {
    data: [u8; JSON_BUF_SIZE],
}

static mut JSON_BUF: AlignedJsonBuf = AlignedJsonBuf { data: [0u8; JSON_BUF_SIZE] };

type SStr = String<24>;
type StageStr = String<20>;
type NameStr = String<PET_NAME_MAX>;

fn sstr(s: &str) -> SStr {
    let mut out = SStr::new();
    let _ = out.push_str(s);
    out
}

crate::enum_key_pair_option! {
    food_save_key, food_from_key, FoodItem;
    Kibble    => "kibble",
    Cod       => "cod",
    Haddock   => "haddock",
    Trout     => "trout",
    Shrimp    => "shrimp",
    Herring   => "herring",
    Turkey    => "turkey",
    Tuna      => "tuna",
    Salmon    => "salmon",
    Chicken   => "chicken",
    Liver     => "liver",
    Beef      => "beef",
    Lamb      => "lamb",
    Mackerel  => "mackerel",
    Carrots   => "carrots",
    Pumpkin   => "pumpkin",
    Treats    => "treats",
    FishBite  => "fish_bite",
    Eggs      => "eggs",
    Nugget    => "nugget",
    Milk      => "milk",
    ChewStick => "chew_stick",
    Puree     => "puree",
}

crate::enum_key_pair_option! {
    toy_save_key, toy_from_key, ToyVariant;
    String_ => "string",
    Feather => "feather",
    Mouse   => "mouse",
    Ball    => "ball",
    Bubbles => "bubbles",
    Laser   => "laser",
}

crate::enum_key_pair_option! {
    pot_save_key, pot_from_key, PotKind;
    Small   => "small",
    Medium  => "medium",
    Large   => "large",
    Planter => "planter",
    Ground  => "ground",
}

crate::enum_key_pair_option! {
    seed_save_key, seed_from_key, SeedKind;
    CatGrass  => "cat_grass",
    Freesia   => "freesia",
    Sunflower => "sunflower",
    Rose      => "rose",
}
crate::enum_key_pair_default! {
    layer_save_key, layer_from_key, PlantLayer;
    default: PlantLayer::Midground;
    Background => "background" | "bg",
    Midground  => "midground" | "mg",
    Foreground => "foreground" | "fg",
}

fn scene_save_key(s: SceneId) -> &'static str {
    match s {
        SceneId::Inside => "inside",
        SceneId::Outside => "outside",
        SceneId::Bedroom => "bedroom",
        SceneId::Kitchen => "kitchen",
        SceneId::Treehouse => "treehouse",
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

crate::enum_key_pair_default! {
    stage_save_key, stage_from_key, PlantStage;
    default: PlantStage::Young;
    EmptyPot        => "empty_pot",
    Seedling        => "seedling",
    Young           => "young",
    Growing         => "growing",
    Mature          => "mature",
    Thriving        => "thriving",
    SeedlingWilted  => "seedling_wilted",
    YoungWilted     => "young_wilted" | "withering",
    GrowingWilted   => "growing_wilted",
    MatureWilted    => "mature_wilted",
    ThrivingWilted  => "thriving_wilted",
    SeedlingDead    => "seedling_dead",
    YoungDead       => "young_dead",
    GrowingDead     => "growing_dead",
    MatureDead      => "mature_dead",
    ThrivingDead    => "thriving_dead",
    Dead            => "dead",
    Dormant         => "dormant",
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
        _ => u8::from_str(s).unwrap_or(0),
    }
}

crate::enum_key_pair_option! {
    gender_save_key, gender_from_key, PetGender;
    Tom   => "tom",
    Queen => "queen",
}

crate::enum_key_pair_option! {
    fav_weather_save_key, fav_weather_from_key, FavWeather;
    Sunny    => "sunny",
    Rainy    => "rainy",
    Snowy    => "snowy",
    Overcast => "overcast",
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

#[derive(Default, Serialize, Deserialize)]
struct WifiEntryData {
    #[serde(default, rename = "b")]
    bssid: heapless::String<17>,
    #[serde(default, rename = "s")]
    ssid: heapless::String<WIFI_SSID_MAX>,
    #[serde(default, rename = "n")]
    count: f32,
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
    #[serde(default)] wifi_familiar: Vec<WifiEntryData, WIFI_FAMILIAR_MAX>,
    #[serde(default)] wifi_recent: Vec<WifiEntryData, WIFI_RECENT_MAX>,
    #[serde(default)] pet_name: Option<NameStr>,
    #[serde(default)] friends: StubMap,
    #[serde(default)] recent_meals: Vec<SStr, RECENT_HISTORY>,
    #[serde(default)] milestones: MilestonesData,
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
    #[serde(default)] time_speed: f32,
    #[serde(default)] coins: i32,
}

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
            seed_type: p.seed.map(|s| sstr(seed_save_key(s))).unwrap_or_default(),
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
        fav_toy: ctx.fav_toy.map(|f| sstr(toy_save_key(f))),
        least_fav_toy: ctx.least_fav_toy.map(|f| sstr(toy_save_key(f))),
        fav_location: ctx.fav_location.map(|f| sstr(fav_location_save_key(f))),
        least_fav_location: ctx.least_fav_location.map(|f| sstr(fav_location_save_key(f))),
        wifi_familiar: build_wifi_list(&ctx.wifi_familiar),
        wifi_recent: build_wifi_list(&ctx.wifi_recent),
        pet_name: {
            if ctx.pet_name.is_empty() {
                None
            } else {
                let mut n = NameStr::new();
                for c in ctx.pet_name.chars() {
                    let _ = n.push(c);
                }
                Some(n)
            }
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
        time_speed: ctx.time_speed,
        coins: ctx.coins,
    }
}
fn find_f32(json: &str, key: &str) -> f32 {
    let pattern = match json.find(key) {
        Some(idx) => &json[idx..],
        None => return 0.0,
    };
    let start = match pattern.find(':') {
        Some(idx) => idx + 1,
        None => return 0.0,
    };
    let end = pattern.find(|c: char| c == ',' || c == '}' || c == ']').unwrap_or(pattern.len());
    let val_str = pattern[start..end].trim();
    f32::from_str(val_str).unwrap_or(0.0)
}

fn find_i32(json: &str, key: &str) -> i32 {
    let pattern = match json.find(key) {
        Some(idx) => &json[idx..],
        None => return 0,
    };
    let start = match pattern.find(':') {
        Some(idx) => idx + 1,
        None => return 0,
    };
    let end = pattern.find(|c: char| c == ',' || c == '}' || c == ']').unwrap_or(pattern.len());
    let val_str = pattern[start..end].trim();
    i32::from_str(val_str).unwrap_or(0)
}

fn apply(data_str: &str, ctx: &mut GameContext) {
    ctx.fullness = find_f32(data_str, "\"fullness\"");
    ctx.energy = find_f32(data_str, "\"energy\"");
    ctx.comfort = find_f32(data_str, "\"comfort\"");
    ctx.playfulness = find_f32(data_str, "\"playfulness\"");
    ctx.focus = find_f32(data_str, "\"focus\"");
    ctx.fulfillment = find_f32(data_str, "\"fulfillment\"");
    ctx.cleanliness = find_f32(data_str, "\"cleanliness\"");
    ctx.curiosity = find_f32(data_str, "\"curiosity\"");
    ctx.sociability = find_f32(data_str, "\"sociability\"");
    ctx.intelligence = find_f32(data_str, "\"intelligence\"");
    ctx.maturity = find_f32(data_str, "\"maturity\"");
    ctx.affection = find_f32(data_str, "\"affection\"");
    ctx.fitness = find_f32(data_str, "\"fitness\"");
    ctx.serenity = find_f32(data_str, "\"serenity\"");
    ctx.courage = find_f32(data_str, "\"courage\"");
    ctx.loyalty = find_f32(data_str, "\"loyalty\"");
    ctx.mischievousness = find_f32(data_str, "\"mischievousness\"");
    
    ctx.zoomies_high_score = find_i32(data_str, "\"zoomies_high_score\"");
    ctx.maze_best_time = find_i32(data_str, "\"maze_best_time\"");
    ctx.snake_high_score = find_i32(data_str, "\"snake_high_score\"");
    ctx.memory_best_score = find_i32(data_str, "\"memory_best_score\"");
    ctx.time_speed = find_f32(data_str, "\"time_speed\"");
    ctx.coins = find_i32(data_str, "\"coins\"");

    ctx.food_stock = [0u8; FOOD_ITEM_COUNT];
    ctx.food_stock[FoodItem::Kibble as usize] = find_i32(data_str, "\"kibble\"") as u8;
    ctx.food_stock[FoodItem::Cod as usize] = find_i32(data_str, "\"cod\"") as u8;
    ctx.food_stock[FoodItem::Haddock as usize] = find_i32(data_str, "\"haddock\"") as u8;
    ctx.food_stock[FoodItem::Trout as usize] = find_i32(data_str, "\"trout\"") as u8;
    ctx.food_stock[FoodItem::Shrimp as usize] = find_i32(data_str, "\"shrimp\"") as u8;
    ctx.food_stock[FoodItem::Herring as usize] = find_i32(data_str, "\"herring\"") as u8;
    ctx.food_stock[FoodItem::Turkey as usize] = find_i32(data_str, "\"turkey\"") as u8;
    ctx.food_stock[FoodItem::Tuna as usize] = find_i32(data_str, "\"tuna\"") as u8;
    ctx.food_stock[FoodItem::Salmon as usize] = find_i32(data_str, "\"salmon\"") as u8;
    ctx.food_stock[FoodItem::Chicken as usize] = find_i32(data_str, "\"chicken\"") as u8;
    ctx.food_stock[FoodItem::Liver as usize] = find_i32(data_str, "\"liver\"") as u8;
    ctx.food_stock[FoodItem::Beef as usize] = find_i32(data_str, "\"beef\"") as u8;
    ctx.food_stock[FoodItem::Lamb as usize] = find_i32(data_str, "\"lamb\"") as u8;
    ctx.food_stock[FoodItem::Mackerel as usize] = find_i32(data_str, "\"mackerel\"") as u8;
    ctx.food_stock[FoodItem::Carrots as usize] = find_i32(data_str, "\"carrots\"") as u8;
    ctx.food_stock[FoodItem::Pumpkin as usize] = find_i32(data_str, "\"pumpkin\"") as u8;
    ctx.food_stock[FoodItem::Treats as usize] = find_i32(data_str, "\"treats\"") as u8;
    ctx.food_stock[FoodItem::FishBite as usize] = find_i32(data_str, "\"fish_bite\"") as u8;
    ctx.food_stock[FoodItem::Eggs as usize] = find_i32(data_str, "\"eggs\"") as u8;
    ctx.food_stock[FoodItem::Nugget as usize] = find_i32(data_str, "\"nugget\"") as u8;
    ctx.food_stock[FoodItem::Milk as usize] = find_i32(data_str, "\"milk\"") as u8;
    ctx.food_stock[FoodItem::ChewStick as usize] = find_i32(data_str, "\"chew_stick\"") as u8;
    ctx.food_stock[FoodItem::Puree as usize] = find_i32(data_str, "\"puree\"") as u8;

    ctx.pots[PotSize::Small as usize] = find_i32(data_str, "\"small\"") as u8;
    ctx.pots[PotSize::Medium as usize] = find_i32(data_str, "\"medium\"") as u8;
    ctx.pots[PotSize::Large as usize] = find_i32(data_str, "\"large\"") as u8;
    ctx.pots[PotSize::Planter as usize] = find_i32(data_str, "\"planter\"") as u8;

    ctx.seeds[SeedKind::CatGrass as usize] = find_i32(data_str, "\"cat_grass\"") as u8;
    ctx.seeds[SeedKind::Sunflower as usize] = find_i32(data_str, "\"sunflower\"") as u8;
    ctx.seeds[SeedKind::Rose as usize] = find_i32(data_str, "\"rose\"") as u8;
    ctx.seeds[SeedKind::Freesia as usize] = find_i32(data_str, "\"freesia\"") as u8;

    ctx.fertilizer = find_i32(data_str, "\"fertilizer\"") as u8;
    ctx.medicine = find_i32(data_str, "\"medicine\"") as u8;
    ctx.sickness = find_f32(data_str, "\"sickness\"");

    ctx.first_impressions = false;
    ctx.recompute_health();
}
pub fn has_save() -> bool {
    storage::has_save()
}

pub fn load(ctx: &mut GameContext) -> bool {
    let buf = unsafe { &mut JSON_BUF.data };
    buf.fill(0);

    let Some(len) = storage::read_latest(buf) else {
        println!("[Save] Новых записей сохранения не обнаружено.");
        return false;
    };

    if len == 0 {
        return false;
    }

    let json_str = match core::str::from_utf8(&buf[..len]) {
        Ok(s) => s,
        Err(_) => {
            println!("[Save] Ошибка: Сейв содержит некорректный UTF-8.");
            return false;
        }
    };

    println!("[Save] Буфер выровнен, восстанавливаем кота из JSON...");
    apply(json_str, ctx);
    true
}
fn build_wifi_list<const N: usize>(src: &heapless::Vec<WifiEntry, N>) -> Vec<WifiEntryData, N> {
    let mut out: Vec<WifiEntryData, N> = Vec::new();
    for e in src.iter() {
        let _ = out.push(WifiEntryData {
            bssid: wifi_tracker::format_bssid(&e.bssid),
            ssid: e.ssid.clone(),
            count: e.count,
        });
    }
    out
}

pub fn save(ctx: &GameContext) -> bool {
    let data = build(ctx);
    let buf = unsafe { &mut JSON_BUF.data };

    match serde_json_core::to_slice(&data, buf) {
        Ok(len) => {
            println!("[Save] Запись сохранения во флеш ({} байт)...", len);
            if storage::write_next(&buf[..len]) {
                unsafe { LAST_SAVE = Some(Instant::now()); }
                true
            } else {
                println!("[Save] Ошибка записи во флеш.");
                false
            }
        }
        Err(e) => {
            println!("[Save] Ошибка сериализации: {:?}", e);
            false
        }
    }
}

static mut LAST_SAVE: Option<Instant> = None;

fn last_save_time() -> Option<Instant> {
    unsafe { LAST_SAVE }
}

pub fn save_if_needed(ctx: &GameContext) {
    let now = Instant::now();
    let should_save = match last_save_time() {
        Some(last) => (now - last) >= SAVE_INTERVAL,
        None => true,
    };

    if should_save {
        if save(ctx) {
            unsafe { LAST_SAVE = Some(now); }
        }
    }
}
