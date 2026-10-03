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

// Сетка из 12 столбцов идеально подходит под русский алфавит на экране 128x64
const FULL_COLS: usize = 12;
const FULL_CELL_W: i32 = 10; // 128 / 12 = 10 пикселей на одну стандартную букву

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

// Полная русская раскладка (строчные буквы и цифры): 12 столбцов х 4 строки
const FULL_LOWER: &[char] = &[
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', KEY_BACK, KEY_DONE,
    'а', 'б', 'в', 'г', 'д', 'е', 'ё', 'ж', 'з', 'и', 'й', 'к',
    'л', 'м', 'н', 'о', 'п', 'р', 'с', 'т', 'у', 'ф', 'х', 'ц',
    'ч', 'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я', ' ', '.', KEY_SHIFT,
];

// Полная русская раскладка (ЗАГЛАВНЫЕ)
const FULL_UPPER: &[char] = &[
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', KEY_BACK, KEY_DONE,
    'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ё', 'Ж', 'З', 'И', 'Й', 'К',
    'Л', 'М', 'Н', 'О', 'П', 'Р', 'С', 'Т', 'У', 'Ф', 'Х', 'Ц',
    'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я', ' ', ',', KEY_SHIFT,
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

// Выделяем 64 байта под строку ввода (запас под двухбайтовую кириллицу UTF-8)
pub const KB_MAX_TEXT: usize = 64;

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
        for c in initial.chars() {
            if self.text.len() + c.len_utf8() > self.max_len {
                break;
            }
            let _ = self.text.push(c);
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
                        if self.text.len() + c.len_utf8() <= self.max_len {
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

        // Безопасное формирование строки предпросмотра
        let mut preview: String<{ KB_MAX_TEXT + 4 }> = String::new();
        let _ = preview.push_str(self.text.as_str());
        let _ = preview.push('_');

        // Unicode-безопасная обрезка текста для OLED экрана
        let char_count = preview.chars().count();
        let display = if char_count > 21 {
            let byte_idx = preview.char_indices().map(|(idx, _)| idx).nth(21).unwrap_or(preview.len());
            &preview.as_str()[..byte_idx]
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

            // Динамическая ширина под спецклавиши для визуального баланса
            let w = match ch {
                KEY_DONE => 18,
                KEY_BACK => 14,
                KEY_SHIFT => 14,
                _ => cell_w,
            };

            if selected || shift_active {
                renderer.draw_rect(Point::new(x, y), Size::new(w as u32, CELL_H as u32), true);
            }

            match ch {
                KEY_BACK => {
                    let ix = x + (w - ICON_W as i32) / 2;
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
                    let ix = x + (w - ICON_W as i32) / 2;
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
                        "ОК"
                    } else if ch == ' ' {
                        "_"
                    } else {
                        label_buf = encode_char(ch);
                        // SAFETY: encode_char yields valid UTF-8.
                        let len = label_buf.iter().position(|&b| b == 0).unwrap_or(label_buf.len());
                        unsafe { core::str::from_utf8_unchecked(&label_buf[..len]) }
                    };
                    
                    let text_w = if ch == KEY_DONE { 12 } else { 6 };
                    let tx = x + (w - text_w) / 2;
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
        let _ = self.text.pop();
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

/// Encode a single UTF-8 char into a 4-byte zero-padded buffer for drawing.
fn encode_char(ch: char) -> [u8; 4] {
    let mut buf = [0u8; 4];
    let s = ch.encode_utf8(&mut buf);
    let _ = s;
    buf
}
