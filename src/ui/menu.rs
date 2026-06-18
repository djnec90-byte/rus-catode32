use embedded_graphics::prelude::{Point, Size};
use heapless::{String, Vec};

use crate::{
    assets::icons,
    context::{FoodItem, PotSize, SeedKind, ToolKind, ToyVariant},
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
    scene::SceneId,
};

pub const VISIBLE_ITEMS: usize = 4;
pub const ROW_HEIGHT: i32 = 16;
pub const DEFAULT_CONTENT_WIDTH: i32 = 120;
pub const DEFAULT_SCROLLBAR_X: i32 = 126;
const ICON_X: i32 = 2;
const ICON_TEXT_GAP: i32 = 3;
const ARROW_INSET_FROM_RIGHT: i32 = 10;
const TRACK_HEIGHT: usize = 64;
const MIN_THUMB_HEIGHT: usize = 4;
const MAX_DEPTH: usize = 4;

const CONFIRM_CHARS: usize = 14;
const CONFIRM_LINE_LEN: usize = 16;
const CONFIRM_VISIBLE: usize = 3;
const CONFIRM_MAX_LINES: usize = 8;

#[derive(Clone, Copy)]
pub struct MenuItem {
    pub label: &'static str,
    pub icon: Option<&'static [u8]>,
    pub submenu: Option<&'static [MenuItem]>,
    pub action: Option<MenuAction>,
    pub confirm: Option<&'static str>,
}

#[derive(Clone, Copy)]
pub enum MenuAction {
    Scene(SceneId),
    Store(StoreAction),
}

#[derive(Clone, Copy)]
pub enum ServiceKind {
    Groom,
    Train,
}

#[derive(Clone, Copy)]
pub enum StoreAction {
    BuyFood(FoodItem, u8),
    BuyToy(ToyVariant, u8),
    BuyPot(PotSize, u8),
    BuySeeds(SeedKind, u8),
    BuyTool(ToolKind, u8),
    BuyFertilizer(u8),
    BuyMedicine(u8),
    BuyService(ServiceKind, u8),
    BuyTrip(SceneId, u8),
    Leave,
}

pub enum MenuResult {
    Continue,
    Closed,
    Action(MenuAction),
}

#[derive(Clone, Copy)]
struct Frame {
    items: &'static [MenuItem],
    selected: usize,
    scroll: usize,
}

struct ConfirmState {
    action: MenuAction,
    lines: Vec<String<CONFIRM_LINE_LEN>, CONFIRM_MAX_LINES>,
    scroll: usize,
}

pub struct Menu {
    current: Frame,
    stack: Vec<Frame, MAX_DEPTH>,
    content_width: i32,
    scrollbar_x: i32,
    confirm: Option<ConfirmState>,
}

impl Menu {
    pub fn new(items: &'static [MenuItem]) -> Self {
        Self::with_width(items, DEFAULT_CONTENT_WIDTH, DEFAULT_SCROLLBAR_X)
    }

    pub fn with_width(
        items: &'static [MenuItem],
        content_width: i32,
        scrollbar_x: i32,
    ) -> Self {
        Self {
            current: Frame { items, selected: 0, scroll: 0 },
            stack: Vec::new(),
            content_width,
            scrollbar_x,
            confirm: None,
        }
    }

    /// Reset to the root menu (clears any submenu stack and pending confirmation).
    pub fn reset_to(&mut self, items: &'static [MenuItem]) {
        self.current = Frame { items, selected: 0, scroll: 0 };
        self.stack.clear();
        self.confirm = None;
    }

    pub fn handle_input(&mut self, buttons: &mut Buttons) -> MenuResult {
        if let Some(confirm) = self.confirm.as_mut() {
            if buttons.was_just_pressed(Button::A) {
                let action = confirm.action;
                self.confirm = None;
                return MenuResult::Action(action);
            }
            if buttons.was_just_pressed(Button::B) {
                self.confirm = None;
                return MenuResult::Continue;
            }
            let max_scroll = confirm.lines.len().saturating_sub(CONFIRM_VISIBLE);
            if buttons.was_just_pressed(Button::Up) && confirm.scroll > 0 {
                confirm.scroll -= 1;
            }
            if buttons.was_just_pressed(Button::Down) && confirm.scroll < max_scroll {
                confirm.scroll += 1;
            }
            return MenuResult::Continue;
        }

        if buttons.was_just_pressed(Button::Menu1) || buttons.was_just_pressed(Button::Menu2) {
            return MenuResult::Closed;
        }

        if buttons.was_just_pressed(Button::Up) && self.current.selected > 0 {
            self.current.selected -= 1;
            self.adjust_scroll();
        }
        if buttons.was_just_pressed(Button::Down)
            && self.current.selected + 1 < self.current.items.len()
        {
            self.current.selected += 1;
            self.adjust_scroll();
        }

        if buttons.was_just_pressed(Button::B) {
            if self.stack.is_empty() {
                return MenuResult::Closed;
            }
            self.exit_submenu();
        }
        if buttons.was_just_pressed(Button::Left) && !self.stack.is_empty() {
            self.exit_submenu();
        }

        if buttons.was_just_pressed(Button::Right) {
            if let Some(item) = self.current.items.get(self.current.selected) {
                if let Some(submenu) = item.submenu {
                    self.enter_submenu(submenu);
                }
            }
        }

        if buttons.was_just_pressed(Button::A) {
            if let Some(item) = self.current.items.get(self.current.selected) {
                if let Some(submenu) = item.submenu {
                    self.enter_submenu(submenu);
                } else if let Some(action) = item.action {
                    if let Some(text) = item.confirm {
                        self.open_confirm(action, text);
                    } else {
                        return MenuResult::Action(action);
                    }
                }
            }
        }

        MenuResult::Continue
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        let visible_end = (self.current.scroll + VISIBLE_ITEMS).min(self.current.items.len());
        for (i, item) in self.current.items[self.current.scroll..visible_end]
            .iter()
            .enumerate()
        {
            let y = (i as i32) * ROW_HEIGHT;
            let actual = self.current.scroll + i;
            let selected = actual == self.current.selected;
            self.draw_item(renderer, item, y, selected);
        }
        self.draw_scrollbar(renderer);

        if let Some(confirm) = self.confirm.as_ref() {
            draw_confirm_dialog(renderer, confirm);
        }
    }

    fn draw_item(&self, renderer: &mut Renderer, item: &MenuItem, y: i32, selected: bool) {
        if selected {
            renderer.draw_rect(
                Point::new(0, y),
                Size::new(self.content_width as u32, ROW_HEIGHT as u32),
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
            renderer.draw_text_inverted(item.label, Point::new(text_x, text_y));
        } else {
            renderer.draw_text(item.label, Point::new(text_x, text_y));
        }

        if item.submenu.is_some() {
            let arrow_x = self.content_width - ARROW_INSET_FROM_RIGHT;
            if selected {
                renderer.draw_text_inverted(">", Point::new(arrow_x, text_y));
            } else {
                renderer.draw_text(">", Point::new(arrow_x, text_y));
            }
        }
    }

    fn draw_scrollbar(&self, renderer: &mut Renderer) {
        let total = self.current.items.len();
        if total <= VISIBLE_ITEMS {
            return;
        }
        let thumb_h = ((TRACK_HEIGHT * VISIBLE_ITEMS) / total).max(MIN_THUMB_HEIGHT);
        let scroll_range = total - VISIBLE_ITEMS;
        let thumb_y = if scroll_range > 0 {
            (self.current.scroll * (TRACK_HEIGHT - thumb_h)) / scroll_range
        } else {
            0
        };
        renderer.draw_rect(
            Point::new(self.scrollbar_x, thumb_y as i32),
            Size::new(2, thumb_h as u32),
            true,
        );
    }

    fn adjust_scroll(&mut self) {
        if self.current.selected < self.current.scroll {
            self.current.scroll = self.current.selected;
        } else if self.current.selected >= self.current.scroll + VISIBLE_ITEMS {
            self.current.scroll = self.current.selected + 1 - VISIBLE_ITEMS;
        }
    }

    fn enter_submenu(&mut self, submenu: &'static [MenuItem]) {
        let _ = self.stack.push(self.current);
        self.current = Frame { items: submenu, selected: 0, scroll: 0 };
    }

    fn exit_submenu(&mut self) {
        if let Some(frame) = self.stack.pop() {
            self.current = frame;
        }
    }

    fn open_confirm(&mut self, action: MenuAction, text: &str) {
        let mut state = ConfirmState {
            action,
            lines: Vec::new(),
            scroll: 0,
        };
        wrap_text(text, CONFIRM_CHARS, &mut state.lines);
        self.confirm = Some(state);
    }
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

fn draw_confirm_dialog(renderer: &mut Renderer, confirm: &ConfirmState) {
    // Outer border + filled-off interior to mirror Python's dialog
    // (rect at 4,12 size 120x40; inner fill at 5,13 size 118x38).
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
