//! Pet interaction menu opened with Menu2 on any location scene. Mirrors the
//! Python `MainScene._build_menu_items` tree: Affection / Train / Feed / Play
//! / Gardening. Items, counts and inventory are recomputed every time the
//! player navigates a page so labels like "Tuna (3)" and "Small pot (2)" stay
//! accurate and out-of-stock entries disappear.

use core::fmt::Write as _;

use embedded_graphics::prelude::{Point, Size};
use heapless::{String, Vec};

use crate::{
    assets::{
        icons,
        plants::{PlantStage, PotKind},
    },
    behavior::{AffectionVariant, AttentionVariant, PlayVariant, TrainingKind},
    context::{FoodItem, GameContext, PotSize, SeedKind},
    input::{Button, Buttons},
    plant_system::{self, Plant},
    render::{Renderer, SpriteOpts},
    scene::SceneId,
    ui::scrollbar::Scrollbar,
};

const LABEL_LEN: usize = 20;
const MAX_PAGE_ITEMS: usize = 16;
const MAX_DEPTH: usize = 4;
const VISIBLE_ITEMS: usize = 4;
const ROW_HEIGHT: i32 = 16;
const CONTENT_WIDTH: i32 = 120;
const SCROLLBAR_X: i32 = 126;
const TRACK_HEIGHT: u32 = 64;
const MIN_THUMB_HEIGHT: u32 = 4;
const ICON_X: i32 = 2;
const ICON_TEXT_GAP: i32 = 3;
const ARROW_INSET_FROM_RIGHT: i32 = 10;

const CONFIRM_CHARS: usize = 14;
const CONFIRM_LINE_LEN: usize = 16;
const CONFIRM_VISIBLE: usize = 3;
const CONFIRM_MAX_LINES: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Root,
    Affection,
    Train,
    Feed,
    FeedMeals,
    FeedSnacks,
    Play,
    Gardening,
    GardeningPlacePot,
    GardeningPlantSeed,
    GardeningPlantSeedInPot,
    GardeningPlantSeedInGround,
    /// Per-plant submenu shown after the player selects a plant via the
    /// PlantSelectionMode cursor. Uses `self.tend_plant_id`.
    GardeningTend,
    GardeningRepot,
    GardeningMove,
    GardeningInspect,
}

#[derive(Clone, Copy)]
pub enum GardeningAction {
    PlacePot(PotKind),
    PlantSeedInPot(SeedKind),
    PlantSeedInGround(SeedKind),
    /// Open the tend cursor — LocationScene starts `PlantSelectionMode` and
    /// re-opens this menu on a dynamic Tend page once the player picks one.
    StartTend,
    Water(u32),
    Fertilize(u32),
    Pluck(u32),
    Repot(u32, PotKind),
    MoveHere(u32),
    MoveTo(u32, SceneId),
    /// No-op: closing the inspect submenu line just dismisses the menu.
    InspectDismiss,
}

#[derive(Clone, Copy)]
pub enum LocationAction {
    Affection(AffectionVariant),
    Attention(AttentionVariant),
    Eat(FoodItem),
    Groom,
    Train(TrainingKind),
    Play(PlayVariant),
    Medicine,
    GoToStore,
    Gardening(GardeningAction),
    /// Vacation scenes inject this as the top menu item; selecting it (with
    /// the "Ready to go home?" confirm) returns the player to the inside scene.
    GoHome,
}

pub enum LocationMenuResult {
    Continue,
    Closed,
    Action(LocationAction),
}

#[derive(Clone)]
struct Item {
    label: String<LABEL_LEN>,
    icon: Option<&'static [u8]>,
    action: Option<LocationAction>,
    submenu: Option<Page>,
    confirm: Option<&'static str>,
}

struct StackFrame {
    page: Page,
    selected: usize,
    scroll: usize,
}

struct ConfirmState {
    action: LocationAction,
    lines: Vec<String<CONFIRM_LINE_LEN>, CONFIRM_MAX_LINES>,
    scroll: usize,
}

pub struct LocationMenu {
    page: Page,
    items: Vec<Item, MAX_PAGE_ITEMS>,
    selected: usize,
    scroll: usize,
    stack: Vec<StackFrame, MAX_DEPTH>,
    confirm: Option<ConfirmState>,
    /// Set by the host scene before opening a tend page so dynamic per-plant
    /// builders know which plant to introspect.
    tend_plant_id: Option<u32>,
    /// Scene that supports plant placement (current scene). Used to filter
    /// the Move submenu so the player can't move to the scene they're in.
    current_scene: Option<SceneId>,
    /// True when the current scene has any PLANT_SURFACES defined; controls
    /// whether the Gardening root entry is shown.
    has_plant_surfaces: bool,
    /// True on vacation scenes; injects the "Go home" item at the top of Root.
    is_vacation: bool,
}

impl LocationMenu {
    pub fn new() -> Self {
        Self {
            page: Page::Root,
            items: Vec::new(),
            selected: 0,
            scroll: 0,
            stack: Vec::new(),
            confirm: None,
            tend_plant_id: None,
            current_scene: None,
            has_plant_surfaces: false,
            is_vacation: false,
        }
    }

    pub fn open(
        &mut self,
        ctx: &GameContext,
        scene: SceneId,
        has_surfaces: bool,
        is_vacation: bool,
    ) {
        self.stack.clear();
        self.confirm = None;
        self.tend_plant_id = None;
        self.current_scene = Some(scene);
        self.has_plant_surfaces = has_surfaces;
        self.is_vacation = is_vacation;
        self.set_page(Page::Root, ctx);
    }

    /// Open directly on the Tend page for a specific plant — used by
    /// LocationScene after the PlantSelectionMode confirms a selection.
    pub fn open_tend(&mut self, ctx: &GameContext, scene: SceneId, plant_id: u32) {
        self.stack.clear();
        self.confirm = None;
        self.tend_plant_id = Some(plant_id);
        self.current_scene = Some(scene);
        self.has_plant_surfaces = true;
        self.is_vacation = false;
        self.set_page(Page::GardeningTend, ctx);
    }

    pub fn handle_input(
        &mut self,
        ctx: &GameContext,
        buttons: &mut Buttons,
    ) -> LocationMenuResult {
        if let Some(confirm) = self.confirm.as_mut() {
            if buttons.was_just_pressed(Button::A) {
                let action = confirm.action;
                self.confirm = None;
                return LocationMenuResult::Action(action);
            }
            if buttons.was_just_pressed(Button::B) {
                self.confirm = None;
                return LocationMenuResult::Continue;
            }
            let max_scroll = confirm.lines.len().saturating_sub(CONFIRM_VISIBLE);
            if buttons.was_just_pressed(Button::Up) && confirm.scroll > 0 {
                confirm.scroll -= 1;
            }
            if buttons.was_just_pressed(Button::Down) && confirm.scroll < max_scroll {
                confirm.scroll += 1;
            }
            return LocationMenuResult::Continue;
        }

        if buttons.was_just_pressed(Button::Menu1) || buttons.was_just_pressed(Button::Menu2) {
            return LocationMenuResult::Closed;
        }

        if buttons.was_just_pressed(Button::Up) && self.selected > 0 {
            self.selected -= 1;
            self.adjust_scroll();
        }
        if buttons.was_just_pressed(Button::Down) && self.selected + 1 < self.items.len() {
            self.selected += 1;
            self.adjust_scroll();
        }

        if buttons.was_just_pressed(Button::B) {
            if self.stack.is_empty() {
                return LocationMenuResult::Closed;
            }
            self.pop_page(ctx);
            return LocationMenuResult::Continue;
        }
        if buttons.was_just_pressed(Button::Left) && !self.stack.is_empty() {
            self.pop_page(ctx);
            return LocationMenuResult::Continue;
        }

        if buttons.was_just_pressed(Button::Right) {
            if let Some(item) = self.items.get(self.selected) {
                if let Some(sub) = item.submenu {
                    self.push_page(sub, ctx);
                    return LocationMenuResult::Continue;
                }
            }
        }

        if buttons.was_just_pressed(Button::A) {
            if let Some(item) = self.items.get(self.selected) {
                if let Some(sub) = item.submenu {
                    self.push_page(sub, ctx);
                    return LocationMenuResult::Continue;
                }
                if let Some(action) = item.action {
                    if let Some(text) = item.confirm {
                        self.open_confirm(action, text);
                    } else {
                        return LocationMenuResult::Action(action);
                    }
                }
            }
        }

        LocationMenuResult::Continue
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        let visible_end = (self.scroll + VISIBLE_ITEMS).min(self.items.len());
        for (i, item) in self.items[self.scroll..visible_end].iter().enumerate() {
            let y = (i as i32) * ROW_HEIGHT;
            let actual = self.scroll + i;
            let selected = actual == self.selected;
            draw_item(renderer, item, y, selected);
        }
        let bar = Scrollbar::new(SCROLLBAR_X, 0, TRACK_HEIGHT, MIN_THUMB_HEIGHT);
        bar.draw(renderer, self.items.len(), VISIBLE_ITEMS, self.scroll);

        if let Some(confirm) = self.confirm.as_ref() {
            draw_confirm_dialog(renderer, confirm);
        }
    }

    fn adjust_scroll(&mut self) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + VISIBLE_ITEMS {
            self.scroll = self.selected + 1 - VISIBLE_ITEMS;
        }
    }

    fn set_page(&mut self, page: Page, ctx: &GameContext) {
        self.page = page;
        self.selected = 0;
        self.scroll = 0;
        self.items.clear();
        build_page(
            page,
            ctx,
            self.current_scene,
            self.has_plant_surfaces,
            self.is_vacation,
            self.tend_plant_id,
            &mut self.items,
        );
    }

    fn push_page(&mut self, page: Page, ctx: &GameContext) {
        let frame = StackFrame {
            page: self.page,
            selected: self.selected,
            scroll: self.scroll,
        };
        let _ = self.stack.push(frame);
        self.set_page(page, ctx);
    }

    fn pop_page(&mut self, ctx: &GameContext) {
        if let Some(frame) = self.stack.pop() {
            self.set_page(frame.page, ctx);
            self.selected = frame.selected.min(self.items.len().saturating_sub(1));
            self.scroll = frame.scroll;
        }
    }

    fn open_confirm(&mut self, action: LocationAction, text: &str) {
        let mut state = ConfirmState {
            action,
            lines: Vec::new(),
            scroll: 0,
        };
        wrap_text(text, CONFIRM_CHARS, &mut state.lines);
        self.confirm = Some(state);
    }
}

fn draw_item(renderer: &mut Renderer, item: &Item, y: i32, selected: bool) {
    if selected {
        renderer.draw_rect(
            Point::new(0, y),
            Size::new(CONTENT_WIDTH as u32, ROW_HEIGHT as u32),
            true,
        );
    }
    let mut text_x = ICON_X;
    if let Some(icon) = item.icon {
        let icon_y = y + (ROW_HEIGHT - icons::ICON_HEIGHT as i32) / 2;
        renderer.draw_sprite_raw(
            icon,
            icons::ICON_WIDTH,
            icons::ICON_HEIGHT,
            Point::new(ICON_X, icon_y),
            SpriteOpts {
                transparent: !selected,
                invert: selected,
                ..Default::default()
            },
        );
        text_x = ICON_X + icons::ICON_WIDTH as i32 + ICON_TEXT_GAP;
    }
    let text_y = y + (ROW_HEIGHT - 10) / 2;
    if selected {
        renderer.draw_text_inverted(item.label.as_str(), Point::new(text_x, text_y));
    } else {
        renderer.draw_text(item.label.as_str(), Point::new(text_x, text_y));
    }
    if item.submenu.is_some() {
        let arrow_x = CONTENT_WIDTH - ARROW_INSET_FROM_RIGHT;
        if selected {
            renderer.draw_text_inverted(">", Point::new(arrow_x, text_y));
        } else {
            renderer.draw_text(">", Point::new(arrow_x, text_y));
        }
    }
}

fn draw_confirm_dialog(renderer: &mut Renderer, confirm: &ConfirmState) {
    renderer.fill_rect_off(Point::new(5, 13), Size::new(118, 38));
    renderer.draw_rect(Point::new(4, 12), Size::new(120, 40), false);

    let total = confirm.lines.len();
    let can_scroll = total > CONFIRM_VISIBLE;
    let visible = total.min(CONFIRM_VISIBLE);
    let y_start = if can_scroll {
        14
    } else {
        14 + (28 - visible as i32 * 8) / 2
    };
    let end = (confirm.scroll + CONFIRM_VISIBLE).min(total);
    for (i, line) in confirm.lines[confirm.scroll..end].iter().enumerate() {
        renderer.draw_text(line.as_str(), Point::new(8, y_start + i as i32 * 8));
    }
    if can_scroll {
        let icon_x = 116;
        if confirm.scroll > 0 {
            renderer.draw_sprite_raw(
                icons::UP_ARROW,
                icons::ARROW_W,
                icons::ARROW_H,
                Point::new(icon_x, 14),
                SpriteOpts::default(),
            );
        }
        if confirm.scroll + CONFIRM_VISIBLE < total {
            renderer.draw_sprite_raw(
                icons::DOWN_ARROW,
                icons::ARROW_W,
                icons::ARROW_H,
                Point::new(icon_x, 32),
                SpriteOpts::default(),
            );
        }
    }
    renderer.draw_text("[A]Yes [B]No", Point::new(20, 42));
}

fn wrap_text(
    text: &str,
    chars_per_line: usize,
    lines: &mut Vec<String<CONFIRM_LINE_LEN>, CONFIRM_MAX_LINES>,
) {
    let chars_per_line = chars_per_line.min(CONFIRM_LINE_LEN);
    for paragraph in text.split('\n') {
        let mut current: String<CONFIRM_LINE_LEN> = String::new();
        for word in paragraph.split(' ') {
            let needs_space = !current.is_empty();
            let extra = (if needs_space { 1 } else { 0 }) + word.len();
            if current.len() + extra <= chars_per_line {
                if needs_space {
                    let _ = current.push(' ');
                }
                let _ = current.push_str(word);
            } else {
                if !current.is_empty() {
                    if lines.push(current.clone()).is_err() {
                        return;
                    }
                    current.clear();
                }
                let _ = current.push_str(&word[..word.len().min(CONFIRM_LINE_LEN)]);
            }
        }
        if lines.push(current).is_err() {
            return;
        }
    }
}

// --- page builders -------------------------------------------------------

fn push_item(
    items: &mut Vec<Item, MAX_PAGE_ITEMS>,
    label: &str,
    icon: Option<&'static [u8]>,
    action: Option<LocationAction>,
    submenu: Option<Page>,
    confirm: Option<&'static str>,
) {
    if items.is_full() {
        return;
    }
    let mut s: String<LABEL_LEN> = String::new();
    let _ = s.push_str(&label[..label.len().min(LABEL_LEN)]);
    let _ = items.push(Item {
        label: s,
        icon,
        action,
        submenu,
        confirm,
    });
}

fn push_count(
    items: &mut Vec<Item, MAX_PAGE_ITEMS>,
    name: &str,
    count: u8,
    icon: Option<&'static [u8]>,
    action: LocationAction,
) {
    if items.is_full() {
        return;
    }
    let mut s: String<LABEL_LEN> = String::new();
    let _ = s.push_str(&name[..name.len().min(LABEL_LEN - 4)]);
    let _ = write!(&mut s, " ({})", count);
    let _ = items.push(Item {
        label: s,
        icon,
        action: Some(action),
        submenu: None,
        confirm: None,
    });
}

fn pot_label(p: PotSize) -> &'static str {
    match p {
        PotSize::Small => "Small pot",
        PotSize::Medium => "Medium pot",
        PotSize::Large => "Large pot",
        PotSize::Planter => "Planter box",
    }
}

fn seed_label(s: SeedKind) -> &'static str {
    match s {
        SeedKind::CatGrass => "Cat Grass",
        SeedKind::Freesia => "Freesia",
        SeedKind::Rose => "Rose",
        SeedKind::Sunflower => "Sunflower",
    }
}

fn scene_label(s: SceneId) -> &'static str {
    match s {
        SceneId::Inside => "To Inside",
        SceneId::Outside => "To Outside",
        SceneId::Bedroom => "To Bedroom",
        SceneId::Kitchen => "To Kitchen",
        SceneId::Treehouse => "To Treehouse",
        _ => "To ???",
    }
}

const ALL_POTS: &[PotSize] = &[PotSize::Small, PotSize::Medium, PotSize::Large, PotSize::Planter];
const ALL_SEEDS: &[SeedKind] = &[
    SeedKind::CatGrass,
    SeedKind::Sunflower,
    SeedKind::Rose,
    SeedKind::Freesia,
];
const PLANTABLE_SCENES: &[SceneId] = &[
    SceneId::Inside,
    SceneId::Outside,
    SceneId::Bedroom,
    SceneId::Kitchen,
    SceneId::Treehouse,
];

fn build_page(
    page: Page,
    ctx: &GameContext,
    current_scene: Option<SceneId>,
    has_surfaces: bool,
    is_vacation: bool,
    tend_plant_id: Option<u32>,
    items: &mut Vec<Item, MAX_PAGE_ITEMS>,
) {
    match page {
        Page::Root => {
            if is_vacation {
                push_item(
                    items,
                    "Go home",
                    Some(icons::HOUSE),
                    Some(LocationAction::GoHome),
                    None,
                    Some("Ready to go home?"),
                );
            }
            push_item(items, "Affection", Some(icons::HEART), None, Some(Page::Affection), None);
            push_item(items, "Train", Some(icons::HAND), None, Some(Page::Train), None);
            push_item(items, "Feed", Some(icons::MEAL), None, Some(Page::Feed), None);
            push_item(items, "Play", Some(icons::TOYS), None, Some(Page::Play), None);
            if has_surfaces {
                push_item(items, "Gardening", Some(icons::TREES), None, Some(Page::Gardening), None);
            }
        }
        Page::Affection => {
            push_item(items, "Pets", Some(icons::HAND), Some(LocationAction::Affection(AffectionVariant::Pets)), None, None);
            push_item(items, "Scratch", Some(icons::HAND), Some(LocationAction::Affection(AffectionVariant::Scratching)), None, None);
            push_item(items, "Kiss", Some(icons::HEART), Some(LocationAction::Affection(AffectionVariant::Kiss)), None, None);
            push_item(items, "Psst psst", Some(icons::HEART_BUBBLE), Some(LocationAction::Attention(AttentionVariant::Psst)), None, None);
            push_item(items, "Groom", Some(icons::HAND), Some(LocationAction::Groom), None, None);
        }
        Page::Train => {
            push_item(items, "Intelligence", Some(icons::HAND), Some(LocationAction::Train(TrainingKind::Intelligence)), None, None);
            push_item(items, "Behavior", Some(icons::HAND), Some(LocationAction::Train(TrainingKind::Behavior)), None, None);
            push_item(items, "Fitness", Some(icons::HAND), Some(LocationAction::Train(TrainingKind::Fitness)), None, None);
            push_item(items, "Sociability", Some(icons::HAND), Some(LocationAction::Train(TrainingKind::Sociability)), None, None);
        }
        Page::Feed => {
            let has_meals = (0..FoodItem::Mackerel as usize + 1)
                .any(|i| ctx.food_stock[i] > 0);
            let has_snacks = (FoodItem::Carrots as usize..FoodItem::Puree as usize + 1)
                .any(|i| ctx.food_stock[i] > 0);
            if has_meals {
                push_item(items, "Meals", Some(icons::MEAL), None, Some(Page::FeedMeals), None);
            }
            if has_snacks {
                push_item(items, "Snacks", Some(icons::KIBBLE), None, Some(Page::FeedSnacks), None);
            }
            if ctx.medicine > 0 {
                let mut label: String<LABEL_LEN> = String::new();
                let _ = write!(&mut label, "Medicine ({})", ctx.medicine);
                let _ = items.push(Item {
                    label,
                    icon: Some(icons::PILL),
                    action: Some(LocationAction::Medicine),
                    submenu: None,
                    confirm: Some("Give medicine?"),
                });
            }
            push_item(items, "Store...", None, Some(LocationAction::GoToStore), None, None);
        }
        Page::FeedMeals => {
            push_food_items(items, ctx, true);
        }
        Page::FeedSnacks => {
            push_food_items(items, ctx, false);
        }
        Page::Play => {
            push_item(
                items,
                "Hand",
                Some(icons::HAND),
                Some(LocationAction::Play(PlayVariant::Hand)),
                None,
                None,
            );
            for toy in ctx.toys.iter() {
                let icon = toy.variant.icon();
                let label = toy.variant.label();
                let action = LocationAction::Play(toy.variant.to_play_variant());
                push_item(items, label, Some(icon), Some(action), None, None);
            }
            push_item(items, "Store...", None, Some(LocationAction::GoToStore), None, None);
        }
        Page::Gardening => {
            let has_any_plants = current_scene
                .map(|s| ctx.plants.iter().any(|p| p.scene == s))
                .unwrap_or(false);
            let has_any_pot = ctx.pots.iter().any(|n| *n > 0);
            let has_any_seed = ctx.seeds.iter().any(|n| *n > 0);
            if has_any_plants {
                push_item(items, "Tend", Some(icons::TREES),
                          Some(LocationAction::Gardening(GardeningAction::StartTend)), None, None);
            }
            if has_any_pot {
                push_item(items, "Place Pot", Some(icons::TREES), None,
                          Some(Page::GardeningPlacePot), None);
            }
            if has_any_seed {
                push_item(items, "Plant Seed", Some(icons::TREES), None,
                          Some(Page::GardeningPlantSeed), None);
            }
            push_item(items, "Store...", None, Some(LocationAction::GoToStore), None, None);
        }
        Page::GardeningPlacePot => {
            for &pot in ALL_POTS {
                let count = ctx.pots[pot as usize];
                if count == 0 {
                    continue;
                }
                push_count(
                    items,
                    pot_label(pot),
                    count,
                    Some(icons::TREES),
                    LocationAction::Gardening(GardeningAction::PlacePot(PotKind::from_pot_size(pot))),
                );
            }
        }
        Page::GardeningPlantSeed => {
            let outside = matches!(current_scene, Some(SceneId::Outside));
            let has_any_seed = ctx.seeds.iter().any(|n| *n > 0);
            if has_any_seed {
                let has_empty_pot = current_scene
                    .map(|s| {
                        ctx.plants
                            .iter()
                            .any(|p| p.scene == s && p.stage == PlantStage::EmptyPot)
                    })
                    .unwrap_or(false);
                if has_empty_pot {
                    push_item(items, "In Pot", Some(icons::TREES), None,
                              Some(Page::GardeningPlantSeedInPot), None);
                }
                if outside {
                    push_item(items, "In Ground", Some(icons::TREES), None,
                              Some(Page::GardeningPlantSeedInGround), None);
                }
            }
        }
        Page::GardeningPlantSeedInPot => {
            for &seed in ALL_SEEDS {
                let count = ctx.seeds[seed as usize];
                if count == 0 {
                    continue;
                }
                push_count(
                    items,
                    seed_label(seed),
                    count,
                    Some(icons::TREES),
                    LocationAction::Gardening(GardeningAction::PlantSeedInPot(seed)),
                );
            }
        }
        Page::GardeningPlantSeedInGround => {
            for &seed in ALL_SEEDS {
                let count = ctx.seeds[seed as usize];
                if count == 0 {
                    continue;
                }
                push_count(
                    items,
                    seed_label(seed),
                    count,
                    Some(icons::TREES),
                    LocationAction::Gardening(GardeningAction::PlantSeedInGround(seed)),
                );
            }
        }
        Page::GardeningTend => {
            let plant = tend_plant_id.and_then(|id| ctx.plants.iter().find(|p| p.id == id));
            let plant = match plant {
                Some(p) => p,
                None => return,
            };
            build_tend_items(items, ctx, plant);
        }
        Page::GardeningInspect => {
            let plant = tend_plant_id.and_then(|id| ctx.plants.iter().find(|p| p.id == id));
            let plant = match plant {
                Some(p) => p,
                None => return,
            };
            for line in plant_system::inspect_lines(plant).iter() {
                push_item(
                    items,
                    line.as_str(),
                    None,
                    Some(LocationAction::Gardening(GardeningAction::InspectDismiss)),
                    None,
                    None,
                );
            }
        }
        Page::GardeningRepot => {
            let plant = tend_plant_id.and_then(|id| ctx.plants.iter().find(|p| p.id == id));
            let plant = match plant {
                Some(p) => p,
                None => return,
            };
            build_repot_items(items, ctx, plant);
        }
        Page::GardeningMove => {
            let plant = tend_plant_id.and_then(|id| ctx.plants.iter().find(|p| p.id == id));
            let (plant, cur_scene) = match (plant, current_scene) {
                (Some(p), Some(s)) => (p, s),
                _ => return,
            };
            push_item(
                items,
                "Around Here",
                None,
                Some(LocationAction::Gardening(GardeningAction::MoveHere(plant.id))),
                None,
                None,
            );
            for &dest in PLANTABLE_SCENES {
                if dest == cur_scene {
                    continue;
                }
                push_item(
                    items,
                    scene_label(dest),
                    None,
                    Some(LocationAction::Gardening(GardeningAction::MoveTo(plant.id, dest))),
                    None,
                    None,
                );
            }
        }
    }
}

fn build_tend_items(items: &mut Vec<Item, MAX_PAGE_ITEMS>, ctx: &GameContext, plant: &Plant) {
    push_item(items, "Inspect", None, None, Some(Page::GardeningInspect), None);

    let alive = plant.stage != PlantStage::EmptyPot && !plant.stage.is_dead();
    if alive {
        push_item(
            items,
            "Water",
            None,
            Some(LocationAction::Gardening(GardeningAction::Water(plant.id))),
            None,
            None,
        );
        if ctx.fertilizer > 0 {
            push_item(
                items,
                "Fertilize",
                None,
                Some(LocationAction::Gardening(GardeningAction::Fertilize(plant.id))),
                None,
                None,
            );
        }
    }

    if plant.pot != PotKind::Ground {
        push_item(items, "Move", None, None, Some(Page::GardeningMove), None);
    }

    // Repot is available when at least one larger pot is in inventory (or, for
    // small/young stages, any other pot).
    if plant.pot != PotKind::Ground && repot_has_options(ctx, plant) {
        push_item(items, "Repot", None, None, Some(Page::GardeningRepot), None);
    }

    let needs_confirm = plant.stage != PlantStage::EmptyPot && !plant.stage.is_dead();
    push_item(
        items,
        "Pluck",
        None,
        Some(LocationAction::Gardening(GardeningAction::Pluck(plant.id))),
        None,
        if needs_confirm { Some("Remove plant?") } else { None },
    );
}

fn pot_rank(p: PotKind) -> i32 {
    match p {
        PotKind::Small => 0,
        PotKind::Medium => 1,
        PotKind::Large => 2,
        PotKind::Planter => 3,
        PotKind::Ground => 4,
    }
}

fn repot_has_options(ctx: &GameContext, plant: &Plant) -> bool {
    let is_large = matches!(plant.stage, PlantStage::Mature | PlantStage::Thriving);
    let cur_rank = pot_rank(plant.pot);
    for &pot in ALL_POTS {
        let kind = PotKind::from_pot_size(pot);
        if kind == plant.pot {
            continue;
        }
        let target_rank = pot_rank(kind);
        if target_rank < cur_rank && is_large {
            continue;
        }
        if ctx.pots[pot as usize] > 0 {
            return true;
        }
    }
    false
}

fn build_repot_items(items: &mut Vec<Item, MAX_PAGE_ITEMS>, ctx: &GameContext, plant: &Plant) {
    let is_large = matches!(plant.stage, PlantStage::Mature | PlantStage::Thriving);
    let cur_rank = pot_rank(plant.pot);
    for &pot in ALL_POTS {
        let kind = PotKind::from_pot_size(pot);
        if kind == plant.pot {
            continue;
        }
        let target_rank = pot_rank(kind);
        if target_rank < cur_rank && is_large {
            continue;
        }
        if ctx.pots[pot as usize] == 0 {
            continue;
        }
        push_item(
            items,
            pot_label(pot),
            None,
            Some(LocationAction::Gardening(GardeningAction::Repot(plant.id, kind))),
            None,
            None,
        );
    }
}

const MEAL_ORDER: &[FoodItem] = &[
    FoodItem::Chicken,
    FoodItem::Salmon,
    FoodItem::Tuna,
    FoodItem::Shrimp,
    FoodItem::Trout,
    FoodItem::Herring,
    FoodItem::Haddock,
    FoodItem::Cod,
    FoodItem::Turkey,
    FoodItem::Beef,
    FoodItem::Lamb,
    FoodItem::Liver,
    FoodItem::Mackerel,
    FoodItem::Kibble,
];

const SNACK_ORDER: &[FoodItem] = &[
    FoodItem::Treats,
    FoodItem::ChewStick,
    FoodItem::Nugget,
    FoodItem::Puree,
    FoodItem::Milk,
    FoodItem::FishBite,
    FoodItem::Eggs,
    FoodItem::Pumpkin,
    FoodItem::Carrots,
];

fn push_food_items(items: &mut Vec<Item, MAX_PAGE_ITEMS>, ctx: &GameContext, meals: bool) {
    let order = if meals { MEAL_ORDER } else { SNACK_ORDER };
    for &item in order {
        let count = ctx.food_stock[item as usize];
        if count == 0 {
            continue;
        }
        let icon = food_icon(item);
        push_count(items, item.label(), count, Some(icon), LocationAction::Eat(item));
    }
}

fn food_icon(item: FoodItem) -> &'static [u8] {
    match item {
        FoodItem::Chicken | FoodItem::Turkey => icons::CHICKEN,
        FoodItem::Cod
        | FoodItem::Haddock
        | FoodItem::Trout
        | FoodItem::Shrimp
        | FoodItem::Herring
        | FoodItem::Tuna
        | FoodItem::Salmon
        | FoodItem::Mackerel
        | FoodItem::FishBite => icons::FISH,
        FoodItem::Beef | FoodItem::Lamb | FoodItem::Liver => icons::MEAL,
        FoodItem::Kibble => icons::KIBBLE,
        _ => icons::KIBBLE,
    }
}

