//! Tiny shared helper for vertical-list scenes (Menu, Settings, LocationMenu).
//!
//! Owns the selected index, the scroll offset, and the logic to keep the
//! selection inside the visible window. Each list-style widget still owns its
//! own drawing, items, and input wiring — `ListNav` only handles the
//! mechanical "Up/Down + scroll" state that was previously duplicated.

#[derive(Clone, Copy, Default)]
pub struct ListNav {
    pub selected: usize,
    pub scroll: usize,
}

impl ListNav {
    pub const fn new() -> Self {
        Self {
            selected: 0,
            scroll: 0,
        }
    }

    pub fn reset(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }

    /// Move selection up one row, scrolling if needed. No-op at the top.
    pub fn up(&mut self, visible: usize) {
        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_scroll(visible);
        }
    }

    /// Move selection down one row, scrolling if needed. No-op at the bottom.
    pub fn down(&mut self, items_len: usize, visible: usize) {
        if self.selected + 1 < items_len {
            self.selected += 1;
            self.adjust_scroll(visible);
        }
    }

    /// Range of item indices currently visible on screen.
    pub fn visible_range(&self, items_len: usize, visible: usize) -> core::ops::Range<usize> {
        self.scroll..(self.scroll + visible).min(items_len)
    }

    fn adjust_scroll(&mut self, visible: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + visible {
            self.scroll = self.selected + 1 - visible;
        }
    }
}
