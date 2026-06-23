use core::fmt::Write;

use embedded_graphics::prelude::{Point, Size};
use heapless::{String, Vec};

use crate::{
    input::{Button, Buttons},
    render::Renderer,
    ui::scrollbar::Scrollbar,
};

pub const MAX_ITEMS: usize = 24;
const VISIBLE_ITEMS: usize = 4;
const ROW_HEIGHT: i32 = 16;
const CONTENT_WIDTH: i32 = 124;
const TRACK_HEIGHT: u32 = 64;
const VALUE_CAP: usize = 12;

#[derive(Clone, Copy)]
pub enum SettingValue {
    Int {
        value: i32,
        min: i32,
        max: i32,
        step: i32,
    },
    /// Fixed-point numeric: `value`, `min`, `max`, `step` are all stored in
    /// units of `1/divisor`. e.g. divisor=10, value=15 -> displayed as "1.5".
    Fixed {
        value: i32,
        min: i32,
        max: i32,
        step: i32,
        divisor: i32,
    },
    Choice {
        index: usize,
        options: &'static [&'static str],
    },
    /// Action item, no value to cycle. Pressing A while it's selected
    /// closes the value-edit loop with `SettingsResult::Activated(index)`
    /// so the caller can do something custom (open a sub-screen, fire a
    /// destructive op, etc).
    Action,
}

impl SettingValue {
    fn cycle_next(&mut self) {
        match self {
            SettingValue::Int { value, max, step, .. } => {
                *value = (*value + *step).min(*max);
            }
            SettingValue::Fixed { value, max, step, .. } => {
                *value = (*value + *step).min(*max);
            }
            SettingValue::Choice { index, options } => {
                if !options.is_empty() {
                    *index = (*index + 1) % options.len();
                }
            }
            SettingValue::Action => {}
        }
    }

    fn cycle_prev(&mut self) {
        match self {
            SettingValue::Int { value, min, step, .. } => {
                *value = (*value - *step).max(*min);
            }
            SettingValue::Fixed { value, min, step, .. } => {
                *value = (*value - *step).max(*min);
            }
            SettingValue::Choice { index, options } => {
                if !options.is_empty() {
                    *index = (*index + options.len() - 1) % options.len();
                }
            }
            SettingValue::Action => {}
        }
    }

    fn display(&self) -> String<VALUE_CAP> {
        let mut s: String<VALUE_CAP> = String::new();
        match self {
            SettingValue::Int { value, .. } => {
                let _ = write!(&mut s, "{}", value);
            }
            SettingValue::Fixed { value, divisor, .. } => {
                let d = (*divisor).max(1);
                let whole = value / d;
                let frac = (value % d).abs();
                let mut frac_digits = 0;
                let mut m = d - 1;
                while m > 0 {
                    frac_digits += 1;
                    m /= 10;
                }
                let sign = if *value < 0 && whole == 0 { "-" } else { "" };
                let _ = write!(
                    &mut s,
                    "{}{}.{:0width$}",
                    sign,
                    whole,
                    frac,
                    width = frac_digits.max(1)
                );
            }
            SettingValue::Choice { index, options } => {
                if let Some(label) = options.get(*index) {
                    let _ = s.push_str(label);
                }
            }
            SettingValue::Action => {}
        }
        s
    }
}

#[derive(Clone, Copy)]
pub struct SettingItem {
    pub label: &'static str,
    pub value: SettingValue,
}

impl SettingItem {
    pub const fn int(label: &'static str, value: i32, min: i32, max: i32, step: i32) -> Self {
        Self {
            label,
            value: SettingValue::Int { value, min, max, step },
        }
    }

    pub const fn fixed(
        label: &'static str,
        value: i32,
        min: i32,
        max: i32,
        step: i32,
        divisor: i32,
    ) -> Self {
        Self {
            label,
            value: SettingValue::Fixed { value, min, max, step, divisor },
        }
    }

    pub const fn choice(
        label: &'static str,
        index: usize,
        options: &'static [&'static str],
    ) -> Self {
        Self {
            label,
            value: SettingValue::Choice { index, options },
        }
    }

    pub const fn action(label: &'static str) -> Self {
        Self {
            label,
            value: SettingValue::Action,
        }
    }
}

pub enum SettingsResult {
    Continue,
    Closed,
    /// User pressed A on a `SettingValue::Action` row at this index. The
    /// settings panel stays open; the caller decides what to do next
    /// (e.g. open a confirm screen, fire a side effect, etc).
    Activated(usize),
}

/// Reusable settings component: a vertical list of editable items where each
/// item is either a numeric range (Left/Right step by `step`) or a choice
/// among a fixed set of labels (Left/Right cycle). B / Menu close the panel
/// and the caller reads the current values back by index.
pub struct Settings {
    items: Vec<SettingItem, MAX_ITEMS>,
    selected: usize,
    scroll: usize,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            scroll: 0,
        }
    }

    pub fn open(&mut self, items: &[SettingItem]) {
        self.items.clear();
        for item in items {
            if self.items.push(*item).is_err() {
                break;
            }
        }
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn value(&self, index: usize) -> Option<&SettingValue> {
        self.items.get(index).map(|i| &i.value)
    }

    pub fn handle_input(&mut self, buttons: &mut Buttons) -> SettingsResult {
        if buttons.was_just_pressed(Button::Menu1)
            || buttons.was_just_pressed(Button::Menu2)
            || buttons.was_just_pressed(Button::B)
        {
            return SettingsResult::Closed;
        }

        if buttons.was_just_pressed(Button::Up) && self.selected > 0 {
            self.selected -= 1;
            self.adjust_scroll();
        }
        if buttons.was_just_pressed(Button::Down) && self.selected + 1 < self.items.len() {
            self.selected += 1;
            self.adjust_scroll();
        }

        if buttons.was_just_pressed(Button::Right) {
            if let Some(item) = self.items.get_mut(self.selected) {
                item.value.cycle_next();
            }
        }
        if buttons.was_just_pressed(Button::Left) {
            if let Some(item) = self.items.get_mut(self.selected) {
                item.value.cycle_prev();
            }
        }

        if buttons.was_just_pressed(Button::A) {
            if let Some(item) = self.items.get(self.selected) {
                if matches!(item.value, SettingValue::Action) {
                    return SettingsResult::Activated(self.selected);
                }
            }
        }

        SettingsResult::Continue
    }

    fn adjust_scroll(&mut self) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + VISIBLE_ITEMS {
            self.scroll = self.selected + 1 - VISIBLE_ITEMS;
        }
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        let end = (self.scroll + VISIBLE_ITEMS).min(self.items.len());
        for (row, idx) in (self.scroll..end).enumerate() {
            let y = row as i32 * ROW_HEIGHT;
            let selected = idx == self.selected;
            self.draw_row(renderer, &self.items[idx], y, selected);
        }

        let bar = Scrollbar::right_edge(0, TRACK_HEIGHT);
        bar.draw(renderer, self.items.len(), VISIBLE_ITEMS, self.scroll);
    }

    fn draw_row(&self, renderer: &mut Renderer, item: &SettingItem, y: i32, selected: bool) {
        if selected {
            renderer.draw_rect(
                Point::new(0, y),
                Size::new(CONTENT_WIDTH as u32, ROW_HEIGHT as u32),
                true,
            );
        }

        let text_y = y + (ROW_HEIGHT - 8) / 2;
        if selected {
            renderer.draw_text_inverted(item.label, Point::new(2, text_y));
        } else {
            renderer.draw_text(item.label, Point::new(2, text_y));
        }

        let value_str = item.value.display();
        let padding = if selected { 16 } else { 12 };
        let value_x = CONTENT_WIDTH - padding - (value_str.len() as i32) * 6;
        if selected {
            renderer.draw_text_inverted(value_str.as_str(), Point::new(value_x, text_y));
        } else {
            renderer.draw_text(value_str.as_str(), Point::new(value_x, text_y));
        }
    }
}
