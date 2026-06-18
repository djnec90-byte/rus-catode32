use embedded_graphics::prelude::{Point, Size};
use heapless::Vec;

use crate::{
    assets::icons,
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
    scene::SceneId,
};

pub const VISIBLE_ITEMS: usize = 4;
pub const ROW_HEIGHT: i32 = 16;
pub const CONTENT_WIDTH: i32 = 120;
const ICON_X: i32 = 2;
const ICON_TEXT_GAP: i32 = 3;
const ARROW_INSET_FROM_RIGHT: i32 = 10;
const SCROLLBAR_X: i32 = 126;
const TRACK_HEIGHT: usize = 64;
const MIN_THUMB_HEIGHT: usize = 4;
const MAX_DEPTH: usize = 4;

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

pub struct Menu {
    current: Frame,
    stack: Vec<Frame, MAX_DEPTH>,
}

impl Menu {
    pub fn new(items: &'static [MenuItem]) -> Self {
        Self {
            current: Frame { items, selected: 0, scroll: 0 },
            stack: Vec::new(),
        }
    }

    pub fn handle_input(&mut self, buttons: &mut Buttons) -> MenuResult {
        if buttons.was_just_pressed(Button::Menu1) || buttons.was_just_pressed(Button::Menu2) {
            return MenuResult::Closed;
        }

        // TODO: confirmation dialog input handling
        // (Python: A=confirm action, B=cancel dialog, up/down=scroll long messages)

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
                    // TODO: open confirmation dialog when item.confirm is Some
                    return MenuResult::Action(action);
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
    }

    fn draw_item(&self, renderer: &mut Renderer, item: &MenuItem, y: i32, selected: bool) {
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
            renderer.draw_text_inverted(item.label, Point::new(text_x, text_y));
        } else {
            renderer.draw_text(item.label, Point::new(text_x, text_y));
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
            Point::new(SCROLLBAR_X, thumb_y as i32),
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
}
