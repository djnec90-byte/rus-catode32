//! Debug plants scene — механика растениеводства и ухода.

use embedded_graphics::prelude::{Point, Size};
use heapless::{String, Vec};
use crate::t;

use crate::{
    context::{FavWeather, GameContext, PlantLayer, SeedKind, PotSize},
    input::{Button, Buttons},
    render::{Renderer, SpriteOpts},
    scene::{Scene, SceneId},
};

const VISIBLE: usize = 8;
const CHAR_W: i32 = 6;
const FULL_CPL: usize = 20;

// Увеличиваем байтовую емкость строк под кириллицу
const LINE_CAP: usize = 64; 
const LINES_CAP: usize = 32; // Защита стека от Stack Overflow

pub struct DebugPlantsScene {
    lines: Vec<String<LINE_CAP>, LINES_CAP>,
    scroll: usize,
    max_scroll: usize,
    selected_idx: usize,
}

impl DebugPlantsScene {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            scroll: 0,
            max_scroll: 0,
            selected_idx: 0,
        }
    }

    fn build_content(&mut self, ctx: &GameContext) {
        self.lines.clear();
        
        // Кнопки действий
        let mut l1 = String::new(); let _ = l1.push_str(t!("Water"));      // Полить
        let mut l2 = String::new(); let _ = l2.push_str(t!("Move"));       // Переместить
        let mut l3 = String::new(); let _ = l3.push_str(t!("Inspect"));    // Осмотреть
        let _ = self.lines.push(l1);
        let _ = self.lines.push(l2);
        let _ = self.lines.push(l3);
        
        let mut spacer = String::new();
        let _ = self.lines.push(spacer);

        // Динамический вывод статуса почвы и растений (на русском)
        let mut s_buf: String<96> = String::new();
        let _ = s_buf.push_str(t!("Status: "));
        if ctx.in_familiar_location {
            let _ = s_buf.push_str(t!("Moist"));
        } else {
            let _ = s_buf.push_str(t!("Dry"));
        }
        wrap_full(&mut self.lines, s_buf.as_str(), FULL_CPL);

        self.max_scroll = self.lines.len().saturating_sub(VISIBLE);
    }

    fn handle_input(&mut self, buttons: &mut Buttons, ctx: &mut GameContext) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::B) {
            return Some(ctx.last_main_scene);
        }

        if buttons.was_just_pressed(Button::Up) {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            }
            if self.scroll > 0 && self.selected_idx < self.scroll {
                self.scroll -= 1;
            }
        } else if buttons.was_just_pressed(Button::Down) {
            if self.selected_idx < 2 { // Всего 3 интерактивные кнопки (0, 1, 2)
                self.selected_idx += 1;
            }
            if self.selected_idx >= self.scroll + VISIBLE {
                self.scroll += 1;
            }
        } else if buttons.was_just_pressed(Button::A) {
            match self.selected_idx {
                0 => { /* Логика Полить */ },
                1 => { /* Логика Переместить */ },
                2 => { /* Логика Осмотреть */ },
                _ => {}
            }
            self.build_content(ctx);
        }
        None
    }

    fn draw_browsing(&self, renderer: &mut Renderer) {
        let total = self.lines.len();

        for i in 0..VISIBLE {
            let line_idx = self.scroll + i;
            if line_idx >= total {
                break;
            }
            let y = (i * 8) as i32;
            
            // Используем безопасное Unicode-извлечение из вектора строк
            if let Some(line) = self.lines.get(line_idx) {
                let is_button = line_idx < 3;
                let is_selected = line_idx == self.selected_idx;

                if is_button && is_selected {
                    renderer.draw_rect(Point::new(0, y), Size::new(128, 8), true);
                    renderer.draw_text_inverted(line.as_str(), Point::new(4, y));
                } else {
                    renderer.draw_text(line.as_str(), Point::new(4, y));
                }
            }
        }
    }
}

impl Scene for DebugPlantsScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.scroll = 0;
        self.selected_idx = 0;
        self.build_content(ctx);
    }

    fn update(&mut self, ctx: &mut GameContext, buttons: &mut Buttons, _dt: f32) -> Option<SceneId> {
        self.handle_input(buttons, ctx)
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        self.draw_browsing(renderer);
    }
}
// ----------------------------------------------------------------------
// Вспомогательные Unicode-безопасные функции переноса текста
// ----------------------------------------------------------------------

/// Безопасный перенос строк с поддержкой кириллицы (UTF-8)
fn wrap_full(out: &mut Vec<String<LINE_CAP>, LINES_CAP>, text: &str, cpl: usize) {
    let cpl = cpl.min(22); // Запас под ширину экрана SSD1306
    let mut current: String<LINE_CAP> = String::new();
    
    for word in text.split(' ') {
        let word_len = word.chars().count();
        let current_len = current.chars().count();
        let needs_space = !current.is_empty();
        let extra = if needs_space { 1 } else { 0 } + word_len;

        if current_len + extra <= cpl {
            if needs_space {
                let _ = current.push(' ');
            }
            let _ = current.push_str(word);
        } else {
            if word_len > cpl && current.is_empty() {
                let mut chunk = String::new();
                for c in word.chars() {
                    if chunk.chars().count() < cpl - 1 {
                        let _ = chunk.push(c);
                    } else {
                        let _ = chunk.push('-');
                        let _ = out.push(chunk.clone());
                        chunk.clear();
                        let _ = chunk.push(c);
                    }
                }
                current = chunk;
            } else {
                if !current.is_empty() {
                    let _ = out.push(current.clone());
                    current.clear();
                }
                let _ = current.push_str(word);
            }
        }
    }
    if !current.is_empty() {
        let _ = out.push(current);
    }
}
