//! Pet interaction menu opened with Menu2 on any location scene. Mirrors the
//! Python `MainScene._build_menu_items` tree: Affection / Train / Feed / Play
//! and (TODO) Gardening. Items, counts and toy availability are recomputed
//! every time the player navigates, so labels like "Tuna (3)" stay accurate
//! and out-of-stock entries disappear.

use core::fmt::Write as _;

use embedded_graphics::prelude::{Point, Size};
use heapless::{String, Vec};

use crate::{
    assets::icons,
    behavior::{AffectionVariant, AttentionVariant, PlayVariant, TrainingKind},
    context::{FoodItem, GameContext, ToyVariant},
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
    ui::scrollbar::Scrollbar,
};

const LABEL_LEN: usize = 20;
const MAX_PAGE_ITEMS: usize = 16;
const MAX_DEPTH: usize = 3;
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
        }
    }

    pub fn open(&mut self, ctx: &GameContext) {
        self.stack.clear();
        self.confirm = None;
        self.set_page(Page::Root, ctx);
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
        build_page(page, ctx, &mut self.items);
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

fn build_page(page: Page, ctx: &GameContext, items: &mut Vec<Item, MAX_PAGE_ITEMS>) {
    match page {
        Page::Root => {
            push_item(items, "Affection", Some(icons::HEART), None, Some(Page::Affection), None);
            push_item(items, "Train", Some(icons::HAND), None, Some(Page::Train), None);
            push_item(items, "Feed", Some(icons::MEAL), None, Some(Page::Feed), None);
            push_item(items, "Play", Some(icons::TOYS), None, Some(Page::Play), None);
            // TODO(plant_system): add a "Gardening" submenu (Tend / Place Pot /
            // Plant Seed / Store...) once plants are ported.
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
            let has_meals = (0..FoodItem::Lamb as usize + 1)
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
        | FoodItem::FishBite => icons::FISH,
        FoodItem::Beef | FoodItem::Lamb | FoodItem::Liver => icons::MEAL,
        FoodItem::Kibble => icons::KIBBLE,
        _ => icons::KIBBLE,
    }
}
