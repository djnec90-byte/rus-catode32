//! Reusable yes/no confirmation dialog.
//!
//! Mirrors the visual style of the Python project's confirm prompts: a 120×40
//! framed box anchored at (4, 12) with an inner fill so it sits cleanly on top
//! of whatever the host scene is drawing. Body text wraps to 14 chars wide
//! and 8 lines max with up to 3 visible at a time (vertical scroll arrows
//! appear when the message overflows).
//!
//! Controls: A confirms, B cancels, Up/Down scroll long messages.

use embedded_graphics::prelude::{Point, Size};
use heapless::{String, Vec};

use crate::{
    assets::icons,
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
};

const LINE_LEN: usize = 16;
const VISIBLE: usize = 3;
const MAX_LINES: usize = 8;
const WRAP_CHARS: usize = 14;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    /// No A/B yet. Caller keeps drawing the dialog.
    Pending,
    /// User pressed A. The dialog auto-closes; caller should run the action.
    Confirmed,
    /// User pressed B. The dialog auto-closes; caller should drop the action.
    Cancelled,
}

pub struct Confirm {
    lines: Vec<String<LINE_LEN>, MAX_LINES>,
    scroll: usize,
    open: bool,
}

impl Confirm {
    pub const fn new() -> Self {
        Self {
            lines: Vec::new(),
            scroll: 0,
            open: false,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Show the dialog with `text` (word-wrapped to fit the box). Existing
    /// content is discarded.
    pub fn open(&mut self, text: &str) {
        self.lines.clear();
        self.scroll = 0;
        wrap_text(text, WRAP_CHARS, &mut self.lines);
        self.open = true;
    }

    /// Force-close the dialog without firing a result. Useful when the host
    /// scene needs to abort (e.g. on scene change).
    pub fn close(&mut self) {
        self.open = false;
        self.lines.clear();
        self.scroll = 0;
    }

    pub fn handle_input(&mut self, buttons: &mut Buttons) -> ConfirmResult {
        if !self.open {
            return ConfirmResult::Pending;
        }
        if buttons.was_just_pressed(Button::A) {
            self.close();
            return ConfirmResult::Confirmed;
        }
        if buttons.was_just_pressed(Button::B) {
            self.close();
            return ConfirmResult::Cancelled;
        }
        let max_scroll = self.lines.len().saturating_sub(VISIBLE);
        if buttons.was_just_pressed(Button::Up) && self.scroll > 0 {
            self.scroll -= 1;
        }
        if buttons.was_just_pressed(Button::Down) && self.scroll < max_scroll {
            self.scroll += 1;
        }
        ConfirmResult::Pending
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        if !self.open {
            return;
        }
        // Outer border + filled-off interior — same geometry the Python
        // project uses for its confirms.
        renderer.fill_rect_off(Point::new(5, 13), Size::new(118, 38));
        renderer.draw_rect(Point::new(4, 12), Size::new(120, 40), false);

        let total = self.lines.len();
        let can_scroll = total > VISIBLE;
        let visible = total.min(VISIBLE);
        let y_start = if can_scroll {
            14
        } else {
            14 + (28 - visible as i32 * 8) / 2
        };

        let end = (self.scroll + VISIBLE).min(total);
        for (i, line) in self.lines[self.scroll..end].iter().enumerate() {
            renderer.draw_text(line.as_str(), Point::new(8, y_start + i as i32 * 8));
        }

        if can_scroll {
            let icon_x = 116;
            if self.scroll > 0 {
                renderer.draw_sprite_raw(
                    icons::UP_ARROW,
                    icons::ARROW_W,
                    icons::ARROW_H,
                    Point::new(icon_x, 14),
                    SpriteOpts::default(),
                );
            }
            if self.scroll + VISIBLE < total {
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
}

/// Word-wrap `text` into `lines`, breaking on spaces. Each line is at most
/// `chars_per_line` characters (capped at `LINE_LEN`). Newlines in `text`
/// start new wrapped paragraphs.
fn wrap_text(
    text: &str,
    chars_per_line: usize,
    lines: &mut Vec<String<LINE_LEN>, MAX_LINES>,
) {
    let chars_per_line = chars_per_line.min(LINE_LEN);
    for paragraph in text.split('\n') {
        let mut current: String<LINE_LEN> = String::new();
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
                let _ = current.push_str(&word[..word.len().min(chars_per_line)]);
            }
        }
        if lines.push(current).is_err() {
            return;
        }
    }
}
