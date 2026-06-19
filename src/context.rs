use heapless::Vec;

use esp_hal::rng::Rng;

use crate::{
    behavior::BehaviorId,
    pet_seed::{PetGender, StarSign},
    scene::SceneId,
    time_system::{Season, Weather},
};

pub const PET_NAME_MAX: usize = 12;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatId {
    Fullness,
    Energy,
    Comfort,
    Playfulness,
    Focus,
    Fulfillment,
    Cleanliness,
    Intelligence,
    Maturity,
    Affection,
    Fitness,
    Serenity,
    Courage,
    Loyalty,
    Mischievousness,
    Curiosity,
    Sociability,
}

impl StatId {
    fn affected_by_sickness(self) -> bool {
        matches!(
            self,
            StatId::Serenity
                | StatId::Fulfillment
                | StatId::Playfulness
                | StatId::Comfort
                | StatId::Energy
                | StatId::Fitness
                | StatId::Focus
        )
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FavWeather {
    Sunny,
    Rainy,
    Snowy,
    Overcast,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FoodKind {
    Kibble,
    WetFood,
    Treat,
    Fish,
    CaughtSnack,
}

/// Specific food/snack items the player can purchase and keep stock of.
/// Distinct from `FoodKind`, which is the coarse category used for meal variety.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum FoodItem {
    // Meals
    Kibble = 0,
    Cod,
    Haddock,
    Trout,
    Shrimp,
    Herring,
    Turkey,
    Tuna,
    Salmon,
    Chicken,
    Liver,
    Beef,
    Lamb,
    // Snacks
    Carrots,
    Pumpkin,
    Treats,
    FishBite,
    Eggs,
    Nugget,
    Milk,
    ChewStick,
    Puree,
}

pub const FOOD_ITEM_COUNT: usize = 22;

impl FoodItem {
    /// Map a specific food item to the coarse `FoodKind` used by the eating
    /// behavior for its bonus table.
    pub fn kind(self) -> FoodKind {
        match self {
            FoodItem::Kibble => FoodKind::Kibble,
            FoodItem::Cod
            | FoodItem::Haddock
            | FoodItem::Trout
            | FoodItem::Shrimp
            | FoodItem::Herring
            | FoodItem::Tuna
            | FoodItem::Salmon
            | FoodItem::FishBite => FoodKind::Fish,
            FoodItem::Turkey
            | FoodItem::Chicken
            | FoodItem::Liver
            | FoodItem::Beef
            | FoodItem::Lamb => FoodKind::WetFood,
            FoodItem::Treats
            | FoodItem::ChewStick
            | FoodItem::Nugget
            | FoodItem::Eggs => FoodKind::Treat,
            FoodItem::Puree
            | FoodItem::Milk
            | FoodItem::Pumpkin
            | FoodItem::Carrots => FoodKind::CaughtSnack,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FoodItem::Kibble => "Kibble",
            FoodItem::Cod => "Cod",
            FoodItem::Haddock => "Haddock",
            FoodItem::Trout => "Trout",
            FoodItem::Shrimp => "Shrimp",
            FoodItem::Herring => "Herring",
            FoodItem::Turkey => "Turkey",
            FoodItem::Tuna => "Tuna",
            FoodItem::Salmon => "Salmon",
            FoodItem::Chicken => "Chicken",
            FoodItem::Liver => "Liver",
            FoodItem::Beef => "Beef",
            FoodItem::Lamb => "Lamb",
            FoodItem::Carrots => "Carrots",
            FoodItem::Pumpkin => "Pumpkin",
            FoodItem::Treats => "Treats",
            FoodItem::FishBite => "Fish Bite",
            FoodItem::Eggs => "Eggs",
            FoodItem::Nugget => "Nugget",
            FoodItem::Milk => "Milk",
            FoodItem::ChewStick => "Chew Stick",
            FoodItem::Puree => "Puree",
        }
    }

    pub fn is_snack(self) -> bool {
        matches!(
            self,
            FoodItem::Carrots
                | FoodItem::Pumpkin
                | FoodItem::Treats
                | FoodItem::FishBite
                | FoodItem::Eggs
                | FoodItem::Nugget
                | FoodItem::Milk
                | FoodItem::ChewStick
                | FoodItem::Puree
        )
    }
}

/// Per-frame snapshot of player input. LocationScene refreshes this from the
/// current `Buttons` view before invoking the behavior layer so behaviors
/// (currently only Playing) can read held directions and one-shot presses.
#[derive(Clone, Copy, Default)]
pub struct InputSnapshot {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub b: bool,
    pub a_just_pressed: bool,
    pub b_just_pressed: bool,
}

/// What the eating behavior records in `recent_meals`. Mirrors Python's
/// `context.recent_meals` (list of food-type strings) but typed so the variety
/// penalty can compare entries cleanly.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MealEntry {
    Item(FoodItem),
    CaughtSnack,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum ToyVariant {
    String_ = 0,
    Feather,
    Mouse,
    Ball,
    Bubbles,
    Laser,
}

pub const TOY_VARIANT_COUNT: usize = 6;

impl ToyVariant {
    pub const fn max_durability(self) -> u8 {
        match self {
            ToyVariant::String_ => 28,
            ToyVariant::Feather => 28,
            ToyVariant::Mouse => 42,
            ToyVariant::Ball => 42,
            ToyVariant::Bubbles => 35,
            ToyVariant::Laser => 100,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            ToyVariant::String_ => "String",
            ToyVariant::Feather => "Feather",
            ToyVariant::Mouse => "Mouse",
            ToyVariant::Ball => "Yarn Ball",
            ToyVariant::Bubbles => "Bubbles",
            ToyVariant::Laser => "Laser",
        }
    }

    pub fn icon(self) -> &'static [u8] {
        use crate::assets::icons;
        match self {
            ToyVariant::String_ => icons::STRING_ICON,
            ToyVariant::Feather => icons::FEATHER,
            ToyVariant::Mouse => icons::MOUSE,
            ToyVariant::Ball => icons::TOYS,
            ToyVariant::Bubbles => icons::BUBBLES,
            ToyVariant::Laser => icons::LASER,
        }
    }

    pub fn to_play_variant(self) -> crate::behavior::PlayVariant {
        use crate::behavior::PlayVariant;
        match self {
            ToyVariant::String_ => PlayVariant::String,
            ToyVariant::Feather => PlayVariant::Feather,
            ToyVariant::Mouse => PlayVariant::Mouse,
            ToyVariant::Ball => PlayVariant::Ball,
            ToyVariant::Bubbles => PlayVariant::Bubbles,
            ToyVariant::Laser => PlayVariant::Laser,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ToyEntry {
    pub variant: ToyVariant,
    pub durability: u8,
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum PotSize {
    Small = 0,
    Medium,
    Large,
    Planter,
}

pub const POT_SIZE_COUNT: usize = 4;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum SeedKind {
    CatGrass = 0,
    Freesia,
    Sunflower,
    Rose,
}

pub const SEED_KIND_COUNT: usize = 4;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum ToolKind {
    Spade = 0,
    WateringCan,
}

pub const TOOL_COUNT: usize = 2;

pub const RECENT_HISTORY: usize = 5;

#[allow(dead_code)]
pub struct GameContext {
    pub health: f32,
    pub fullness: f32,
    pub energy: f32,
    pub comfort: f32,
    pub playfulness: f32,
    pub focus: f32,

    pub fulfillment: f32,
    pub cleanliness: f32,
    pub intelligence: f32,
    pub maturity: f32,
    pub affection: f32,

    pub fitness: f32,
    pub serenity: f32,

    pub courage: f32,
    pub loyalty: f32,
    pub mischievousness: f32,
    pub curiosity: f32,
    pub sociability: f32,

    pub sickness: f32,

    pub time_speed: f32,

    pub coins: i32,

    // Inventory — separate per-category storage. Counts are indexed by enum
    // discriminant (use `as usize`). Toys are a sparse list with per-instance
    // durability since each variant has at most one entry in the player's bag.
    pub food_stock: [u8; FOOD_ITEM_COUNT],
    pub toys: Vec<ToyEntry, TOY_VARIANT_COUNT>,
    pub pots: [u8; POT_SIZE_COUNT],
    pub seeds: [u8; SEED_KIND_COUNT],
    pub tools: [bool; TOOL_COUNT],
    pub fertilizer: u8,
    pub medicine: u8,
    /// Set when the player administers a dose; cleared by a sleep/nap cycle.
    /// Drives the healing tick during the next rest.
    pub medicine_pending: bool,

    pub zoomies_high_score: i32,
    pub maze_best_time: i32,
    pub snake_high_score: i32,
    pub memory_best_score: i32,
    pub hanjie_best_time: i32,

    // World/environment state — advanced by TimeSystem each frame.
    pub time_hours: u8,
    pub time_minutes: u8,
    pub day_number: u32,
    pub season_offset: u16,
    pub season: Season,
    pub moon_phase: u8,
    pub weather: Weather,
    pub temperature: f32,
    pub weather_step: u32,
    pub weather_timer: f32,
    pub meteor_shower_timer: f32,

    pub last_main_scene: SceneId,

    // Behavior-system state.
    pub recent_behaviors: Vec<BehaviorId, RECENT_HISTORY>,
    pub current_behavior_name: Option<&'static str>,
    pub pending_wake_greeting: bool,

    // Scene-supplied bounds and props (set by scenes during enter()).
    pub scene_x_min: i32,
    pub scene_x_max: i32,
    pub cat_bed_x: Option<i32>,
    pub in_cat_bed: bool,
    /// Foreground-layer camera offset, refreshed by LocationScene each frame
    /// so behaviors can convert character world-x into screen-x without
    /// plumbing the environment through every call.
    pub scene_camera_x: i32,

    /// Per-frame input view set by the active scene before behavior updates
    /// so behaviors (currently only Playing) can react to held directions
    /// and one-shot B/A presses.
    pub input: InputSnapshot,

    // RNG seed used by the behavior layer.
    pub rng: u32,
    // Hardware RNG handle for code paths that need fresh entropy (e.g. adoption
    // seed generation). Carries no state — `Rng` is a zero-sized peripheral
    // marker; it's stashed on the context so scenes can reach it without
    // plumbing a separate parameter.
    pub hw_rng: Rng,

    // Cross-scene signals.
    pub pending_scene: Option<SceneId>,
    pub pending_popup_icon: Option<&'static str>,

    // TODO(plant_system): drive from real plant inventory once ported.
    pub scene_plant_health: i8,
    pub in_familiar_location: bool,

    // Pet identity — set during the adoption scene from a 64-bit seed.
    pub pet_seed: u64,
    pub pet_name: heapless::String<PET_NAME_MAX>,
    pub pet_gender: Option<PetGender>,
    pub star_sign: Option<StarSign>,

    // Personality-derived favorites. All set during adoption from `pet_seed`.
    pub fav_weather: Option<FavWeather>,
    pub fav_meal: Option<FoodItem>,
    pub least_fav_meal: Option<FoodItem>,
    pub fav_snack: Option<FoodItem>,
    pub least_fav_snack: Option<FoodItem>,
    pub fav_toy: Option<ToyVariant>,
    pub least_fav_toy: Option<ToyVariant>,
    pub fav_location: Option<SceneId>,
    pub least_fav_location: Option<SceneId>,

    /// Last few meals (per-item granularity). Drives the eating-behavior
    /// variety penalty and the snack-streak sickness ramp.
    pub recent_meals: Vec<MealEntry, RECENT_HISTORY>,

    // First-run tutorial state.
    pub first_impressions: bool,
    // Milestones tracked by interaction behaviors for first-time bonuses.
    pub milestone_fed: bool,
    pub milestone_petted: bool,
    pub milestone_played: bool,
    pub milestone_groomed: bool,
    pub milestone_store: bool,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            health: 50.0,
            fullness: 50.0,
            energy: 50.0,
            comfort: 50.0,
            playfulness: 50.0,
            focus: 50.0,
            fulfillment: 50.0,
            cleanliness: 50.0,
            intelligence: 50.0,
            maturity: 50.0,
            affection: 50.0,
            fitness: 50.0,
            serenity: 50.0,
            courage: 50.0,
            loyalty: 50.0,
            mischievousness: 50.0,
            curiosity: 50.0,
            sociability: 50.0,
            sickness: 0.0,
            time_speed: 1.0,
            coins: 50,
            food_stock: {
                let mut s = [0u8; FOOD_ITEM_COUNT];
                s[FoodItem::Kibble as usize] = 5;
                s[FoodItem::Nugget as usize] = 3;
                s
            },
            toys: Vec::new(),
            pots: [0u8; POT_SIZE_COUNT],
            seeds: [0u8; SEED_KIND_COUNT],
            tools: [false; TOOL_COUNT],
            fertilizer: 0,
            medicine: 0,
            medicine_pending: false,
            zoomies_high_score: 0,
            maze_best_time: 0,
            snake_high_score: 0,
            memory_best_score: -1,
            hanjie_best_time: -1,

            time_hours: 12,
            time_minutes: 0,
            day_number: 0,
            // TODO: derive season_offset from ctx.pet_seed once the personality system is ported.
            // Matches Python default: all pets start in late spring.
            season_offset: 120,
            season: Season::Spring,
            moon_phase: 2,
            weather: Weather::Clear,
            temperature: 20.0,
            weather_step: 0,
            weather_timer: 0.0,
            meteor_shower_timer: 0.0,

            last_main_scene: SceneId::Inside,

            recent_behaviors: Vec::new(),
            current_behavior_name: None,
            pending_wake_greeting: false,

            scene_x_min: 10,
            scene_x_max: 118,
            cat_bed_x: None,
            in_cat_bed: false,
            scene_camera_x: 0,
            input: InputSnapshot::default(),

            rng: 0xC0FFEEu32,
            hw_rng: Rng::new(),

            pending_scene: None,
            pending_popup_icon: None,

            scene_plant_health: 0,
            in_familiar_location: true,

            pet_seed: 0,
            pet_name: heapless::String::new(),
            pet_gender: None,
            star_sign: None,

            fav_weather: None,
            fav_meal: None,
            least_fav_meal: None,
            fav_snack: None,
            least_fav_snack: None,
            fav_toy: None,
            least_fav_toy: None,
            fav_location: None,
            least_fav_location: None,

            recent_meals: Vec::new(),

            first_impressions: false,
            milestone_fed: false,
            milestone_petted: false,
            milestone_played: false,
            milestone_groomed: false,
            milestone_store: false,
        }
    }

    pub fn meteor_shower_happening(&self) -> bool {
        self.meteor_shower_timer > 0.0
    }

    pub fn recompute_health(&mut self) {
        let raw = 0.25 * self.fullness
            + 0.20 * self.fitness
            + 0.20 * self.energy
            + 0.15 * self.cleanliness
            + 0.05 * self.comfort
            + 0.05 * self.affection
            + 0.025 * self.fulfillment
            + 0.025 * self.focus
            + 0.025 * self.intelligence
            + 0.025 * self.playfulness;
        self.health = raw.clamp(0.0, 100.0);
    }

    fn stat_mut(&mut self, id: StatId) -> &mut f32 {
        match id {
            StatId::Fullness => &mut self.fullness,
            StatId::Energy => &mut self.energy,
            StatId::Comfort => &mut self.comfort,
            StatId::Playfulness => &mut self.playfulness,
            StatId::Focus => &mut self.focus,
            StatId::Fulfillment => &mut self.fulfillment,
            StatId::Cleanliness => &mut self.cleanliness,
            StatId::Intelligence => &mut self.intelligence,
            StatId::Maturity => &mut self.maturity,
            StatId::Affection => &mut self.affection,
            StatId::Fitness => &mut self.fitness,
            StatId::Serenity => &mut self.serenity,
            StatId::Courage => &mut self.courage,
            StatId::Loyalty => &mut self.loyalty,
            StatId::Mischievousness => &mut self.mischievousness,
            StatId::Curiosity => &mut self.curiosity,
            StatId::Sociability => &mut self.sociability,
        }
    }

    /// Apply a batch of stat changes with asymptotic damping near 0 and 100.
    /// Stats near their ceiling resist further increases; stats near the floor
    /// resist further decreases. Mirrors Python `context.apply_stat_changes`.
    pub fn apply_stat_changes(&mut self, changes: &[(StatId, f32)]) {
        use micromath::F32Ext;
        const EXP: f32 = 0.7;
        let sickness = self.sickness;
        for &(stat, delta) in changes {
            if delta == 0.0 {
                continue;
            }
            let cur = *self.stat_mut(stat);
            let mut d = delta;
            if d > 0.0 {
                let room = ((100.0 - cur) / 100.0).max(0.0);
                d *= room.powf(EXP);
                if stat.affected_by_sickness() {
                    if sickness >= 8.0 {
                        d *= 0.4;
                    } else if sickness >= 5.0 {
                        d *= 0.6;
                    } else if sickness >= 2.0 {
                        d *= 0.8;
                    }
                }
            } else {
                let room = (cur / 100.0).max(0.0);
                d *= room.powf(EXP);
            }
            let new_val = (cur + d).clamp(0.0, 100.0);
            *self.stat_mut(stat) = new_val;
        }
        self.recompute_health();
    }

    /// Push a behavior id onto the recent-history ring; index 0 = most recent.
    pub fn record_behavior(&mut self, id: BehaviorId) {
        if self.recent_behaviors.is_full() {
            self.recent_behaviors.pop();
        }
        let _ = self.recent_behaviors.insert(0, id);
    }

    pub fn record_meal(&mut self, entry: MealEntry) {
        if self.recent_meals.is_full() {
            self.recent_meals.pop();
        }
        let _ = self.recent_meals.insert(0, entry);
    }

    /// Recency index of `id` in the recent-behaviors ring, or None.
    pub fn recent_index(&self, id: BehaviorId) -> Option<usize> {
        self.recent_behaviors.iter().position(|b| *b == id)
    }

    pub fn add_food_stock(&mut self, item: FoodItem, uses: u8) {
        let i = item as usize;
        self.food_stock[i] = self.food_stock[i].saturating_add(uses);
    }

    pub fn add_pot(&mut self, pot: PotSize) {
        let i = pot as usize;
        self.pots[i] = self.pots[i].saturating_add(1);
    }

    pub fn add_seeds(&mut self, seed: SeedKind, n: u8) {
        let i = seed as usize;
        self.seeds[i] = self.seeds[i].saturating_add(n);
    }

    pub fn owns_tool(&self, tool: ToolKind) -> bool {
        self.tools[tool as usize]
    }

    pub fn set_tool(&mut self, tool: ToolKind, owned: bool) {
        self.tools[tool as usize] = owned;
    }

    pub fn find_toy(&self, variant: ToyVariant) -> Option<usize> {
        self.toys.iter().position(|t| t.variant == variant)
    }

    pub fn add_toy(&mut self, variant: ToyVariant) -> bool {
        if self.find_toy(variant).is_some() {
            return false;
        }
        self.toys
            .push(ToyEntry {
                variant,
                durability: variant.max_durability(),
            })
            .is_ok()
    }

    pub fn refresh_toy(&mut self, variant: ToyVariant) -> bool {
        if let Some(idx) = self.find_toy(variant) {
            self.toys[idx].durability = variant.max_durability();
            true
        } else {
            false
        }
    }
}
