//! On-screen keyboard for text/hex entry.

use embedded_graphics::prelude::{Point, Size};
use heapless::String;

use crate::{
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
};

const SEP_Y: i32 = 9;
const GRID_Y: i32 = 11;
const CELL_H: i32 = 13;

const FULL_COLS: usize = 11;
const FULL_CELL_W: i32 = 128 / FULL_COLS as i32; // 11

const HEX_COLS: usize = 9;
const HEX_CELL_W: i32 = 128 / HEX_COLS as i32; // 14

/// Special-key sentinels, chosen out-of-band from normal characters.
const KEY_EMPTY: char = '\x00';
const KEY_BACK: char = '\x08';
const KEY_DONE: char = '\r';
const KEY_SHIFT: char = '\x0e';

const ICON_BACK: &[u8] =
    b"\x1e\x22\x42\x82\x82\x82\x42\x22\x1e";
const ICON_SHIFT: &[u8] =
    b"\x10\x28\x44\x82\xee\x28\x28\x28\x38";
const ICON_W: u16 = 7;
const ICON_H: u16 = 9;

const FULL_LOWER: &[char] = &[
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', KEY_EMPTY,
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
    'w', 'x', 'y', 'z', ' ', '.', ',', KEY_SHIFT, KEY_BACK, KEY_DONE, KEY_EMPTY,
];

const FULL_UPPER: &[char] = &[
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', KEY_EMPTY,
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K',
    'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
    'W', 'X', 'Y', 'Z', ' ', '.', ',', KEY_SHIFT, KEY_BACK, KEY_DONE, KEY_EMPTY,
];

const HEX_CHARS: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8',
    '9', 'A', 'B', 'C', 'D', 'E', 'F', KEY_BACK, KEY_DONE,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Full,
    Hex,
}

pub const KB_MAX_TEXT: usize = 32;

pub struct OnScreenKeyboard {
    charset: Charset,
    max_len: usize,
    text: String<KB_MAX_TEXT>,
    shift: bool,
    cur_row: usize,
    cur_col: usize,
}

impl OnScreenKeyboard {
    pub fn new(charset: Charset, max_len: usize) -> Self {
        Self {
            charset,
            max_len: max_len.min(KB_MAX_TEXT),
            text: String::new(),
            shift: false,
            cur_row: 0,
            cur_col: 0,
        }
    }

    pub fn open(&mut self, initial: &str) {
        self.text.clear();
        for c in initial.chars().take(self.max_len) {
            if self.text.push(c).is_err() {
                break;
            }
        }
        self.shift = false;
        self.cur_row = 0;
        self.cur_col = 0;
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    fn cols(&self) -> usize {
        match self.charset {
            Charset::Full => FULL_COLS,
            Charset::Hex => HEX_COLS,
        }
    }

    fn cell_w(&self) -> i32 {
        match self.charset {
            Charset::Full => FULL_CELL_W,
            Charset::Hex => HEX_CELL_W,
        }
    }

    fn chars(&self) -> &'static [char] {
        match self.charset {
            Charset::Full => {
                if self.shift {
                    FULL_UPPER
                } else {
                    FULL_LOWER
                }
            }
            Charset::Hex => HEX_CHARS,
        }
    }

    fn has_shift(&self) -> bool {
        matches!(self.charset, Charset::Full)
    }

    /// Returns the typed string when the user confirms; None while still editing.
    pub fn handle_input(&mut self, buttons: &mut Buttons) -> Option<String<KB_MAX_TEXT>> {
        let chars = self.chars();
        let cols = self.cols();
        let max_row = (chars.len().saturating_sub(1)) / cols;

        if buttons.was_just_pressed(Button::Up) {
            if self.cur_row > 0 {
                self.cur_row -= 1;
                self.clamp_col(chars, cols);
            }
        } else if buttons.was_just_pressed(Button::Down) {
            if self.cur_row < max_row {
                self.cur_row += 1;
                self.clamp_col(chars, cols);
            }
        } else if buttons.was_just_pressed(Button::Left) {
            if self.cur_col > 0 {
                self.cur_col -= 1;
                self.skip_empty_left(chars, cols);
            } else if self.cur_row > 0 {
                self.cur_row -= 1;
                self.cur_col = cols - 1;
                self.clamp_col(chars, cols);
            }
        } else if buttons.was_just_pressed(Button::Right) {
            let cur_idx = self.cur_row * cols + self.cur_col;
            if cur_idx < chars.len() - 1 {
                if self.cur_col < cols - 1 {
                    self.cur_col += 1;
                } else {
                    self.cur_row += 1;
                    self.cur_col = 0;
                }
                self.skip_empty_right(chars, cols, max_row);
            }
        }

        if buttons.was_just_pressed(Button::B) {
            self.backspace();
            return None;
        }

        if buttons.was_just_pressed(Button::Menu1) || buttons.was_just_pressed(Button::Menu2) {
            return Some(self.text.clone());
        }

        if buttons.was_just_pressed(Button::A) {
            let idx = self.cur_row * cols + self.cur_col;
            if let Some(&key) = chars.get(idx) {
                match key {
                    KEY_DONE => return Some(self.text.clone()),
                    KEY_BACK => self.backspace(),
                    KEY_SHIFT => {
                        if self.has_shift() {
                            self.shift = !self.shift;
                        }
                    }
                    KEY_EMPTY => {}
                    c => {
                        if self.text.len() < self.max_len {
                            let _ = self.text.push(c);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn draw(&self, renderer: &mut Renderer) {
        let chars = self.chars();
        let cols = self.cols();
        let cell_w = self.cell_w();

        // Text input line with trailing caret.
        let mut preview: String<{ KB_MAX_TEXT + 1 }> = String::new();
        let _ = preview.push_str(self.text.as_str());
        let _ = preview.push('_');
        let display = if preview.len() > 21 {
            &preview.as_str()[..21]
        } else {
            preview.as_str()
        };
        renderer.draw_text(display, Point::new(0, 0));
        renderer.draw_line(Point::new(0, SEP_Y), Point::new(127, SEP_Y));

        for (i, &ch) in chars.iter().enumerate() {
            if ch == KEY_EMPTY {
                continue;
            }
            let row = i / cols;
            let col = i % cols;
            let x = (col as i32) * cell_w;
            let y = GRID_Y + (row as i32) * CELL_H;

            let selected = row == self.cur_row && col == self.cur_col;
            let shift_active = ch == KEY_SHIFT && self.shift;

            if selected || shift_active {
                let w = if ch == KEY_DONE { 16 } else { cell_w };
                renderer.draw_rect(Point::new(x, y), Size::new(w as u32, CELL_H as u32), true);
            }

            match ch {
                KEY_BACK => {
                    let ix = x + (cell_w - ICON_W as i32) / 2;
                    let iy = y + (CELL_H - ICON_H as i32) / 2;
                    renderer.draw_sprite_raw(
                        ICON_BACK,
                        ICON_W,
                        ICON_H,
                        Point::new(ix, iy),
                        SpriteOpts {
                            invert: selected,
                            transparent: !selected,
                            transparent_color: false,
                            ..Default::default()
                        },
                    );
                }
                KEY_SHIFT => {
                    let ix = x + (cell_w - ICON_W as i32) / 2;
                    let iy = y + (CELL_H - ICON_H as i32) / 2;
                    let lit = selected || shift_active;
                    renderer.draw_sprite_raw(
                        ICON_SHIFT,
                        ICON_W,
                        ICON_H,
                        Point::new(ix, iy),
                        SpriteOpts {
                            invert: lit,
                            transparent: !lit,
                            transparent_color: false,
                            ..Default::default()
                        },
                    );
                }
                _ => {
                    let label_buf: [u8; 4];
                    let label: &str = if ch == KEY_DONE {
                        "OK"
                    } else if ch == ' ' {
                        "_"
                    } else {
                        label_buf = encode_char(ch);
                        // SAFETY: encode_char yields valid UTF-8.
                        let len = label_buf.iter().position(|&b| b == 0).unwrap_or(label_buf.len());
                        unsafe { core::str::from_utf8_unchecked(&label_buf[..len]) }
                    };
                    let tx = x + (cell_w - 8) / 2;
                    let ty = y + (CELL_H - 8) / 2;
                    if selected || shift_active {
                        renderer.draw_text_inverted(label, Point::new(tx, ty));
                    } else {
                        renderer.draw_text(label, Point::new(tx, ty));
                    }
                }
            }
        }
    }

    fn backspace(&mut self) {
        self.text.pop();
    }

    fn clamp_col(&mut self, chars: &[char], cols: usize) {
        let row_start = self.cur_row * cols;
        let mut max_col = 0;
        for c in 0..cols {
            let idx = row_start + c;
            if idx < chars.len() && chars[idx] != KEY_EMPTY {
                max_col = c;
            }
        }
        if self.cur_col > max_col {
            self.cur_col = max_col;
        }
    }

    fn skip_empty_right(&mut self, chars: &[char], cols: usize, max_row: usize) {
        loop {
            let idx = self.cur_row * cols + self.cur_col;
            if idx >= chars.len() || chars[idx] != KEY_EMPTY {
                break;
            }
            if self.cur_col < cols - 1 {
                self.cur_col += 1;
            } else if self.cur_row < max_row {
                self.cur_row += 1;
                self.cur_col = 0;
            } else {
                break;
            }
        }
    }

    fn skip_empty_left(&mut self, chars: &[char], cols: usize) {
        while self.cur_col > 0 {
            let idx = self.cur_row * cols + self.cur_col;
            if idx >= chars.len() || chars[idx] == KEY_EMPTY {
                self.cur_col -= 1;
            } else {
                break;
            }
        }
    }
}

/// Encode a single ASCII char into a 4-byte zero-padded buffer for drawing.
fn encode_char(ch: char) -> [u8; 4] {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    let _ = s;
    buf
}
