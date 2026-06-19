use embedded_graphics::prelude::{Point, Size};

use crate::render::Renderer;

const THUMB_WIDTH: u32 = 2;

/// Stateless vertical scrollbar. The caller owns the scroll offset and only
/// asks the widget to render a thumb proportional to the current view.
#[derive(Clone, Copy)]
pub struct Scrollbar {
    pub x: i32,
    pub y: i32,
    pub track_height: u32,
    pub min_thumb_height: u32,
}

impl Scrollbar {
    pub const fn new(x: i32, y: i32, track_height: u32, min_thumb_height: u32) -> Self {
        Self { x, y, track_height, min_thumb_height }
    }

    /// 128-wide display default: thumb pinned to the right edge.
    pub const fn right_edge(y: i32, track_height: u32) -> Self {
        Self::new(126, y, track_height, 4)
    }

    pub fn draw(&self, renderer: &mut Renderer, total: usize, visible: usize, scroll: usize) {
        if total <= visible || self.track_height == 0 {
            return;
        }
        let track = self.track_height as usize;
        let min_h = self.min_thumb_height as usize;
        let thumb_h = ((track * visible) / total).max(min_h).min(track);
        let scroll_range = total - visible;
        let thumb_y = if scroll_range > 0 {
            (scroll * (track - thumb_h)) / scroll_range
        } else {
            0
        };
        renderer.draw_rect(
            Point::new(self.x, self.y + thumb_y as i32),
            Size::new(THUMB_WIDTH, thumb_h as u32),
            true,
        );
    }
}
