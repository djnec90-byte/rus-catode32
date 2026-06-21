//! Pipes minigame — rotate pipe pieces to route water from inlet to outlet.

use core::fmt::Write as _;

use embedded_graphics::prelude::{Point, Size};
use heapless::String;

use crate::{
    assets::minigame_assets::{
        PIPE_CORNER, PIPE_FAT, PIPE_OUTLET_LEFT, PIPE_OUTLET_RIGHT, PIPE_STRAIGHT,
    },
    context::{GameContext, StatId},
    input::{Button, Buttons},
    rand::xorshift32,
    render::{Renderer, Sprite, SpriteOpts},
    scene::{Scene, SceneId},
    ui::menu::{Menu, MenuItem, MenuResult},
    ui::popup::Popup,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PipesAction {
    NewBoard,
}

const OPTIONS_ITEMS: &[MenuItem<PipesAction>] = &[
    MenuItem { label: "New Board", icon: None, submenu: None, action: Some(PipesAction::NewBoard), confirm: None, confirm_on_vacation: None },
];

const CELL: i32 = 9;
const PLAY_COLS: usize = 12;
const PLAY_ROWS: usize = 7;
const OUTLET_ROW: usize = 3;

const INLET_COL: usize = 0;
const OUTLET_COL: usize = PLAY_COLS + 1; // 13
const TOTAL_COLS: usize = PLAY_COLS + 2; // 14
const PLAY_TOTAL: usize = PLAY_ROWS * PLAY_COLS;
const FILL_TOTAL: usize = PLAY_ROWS * TOTAL_COLS;

const GRID_X: i32 = 0;
const GRID_Y: i32 = 0;

const FLOW_SPEED_NORMAL: f32 = 6.0;
const FLOW_SPEED_FAT: f32 = 2.0;
const START_DELAY: f32 = 16.0;
// Rise animation fills the full vertical channel (PLAY_ROWS * CELL px) over
// START_DELAY seconds, then halved (matches Python `... / 2.0`).
const INLET_RISE_SPEED: f32 =
    (PLAY_ROWS as f32 * CELL as f32) / START_DELAY / 2.0;
const WIN_DELAY: f32 = 2.5;
const BROKEN_DELAY: f32 = 2.0;

// Directions: L=0, R=1, U=2, D=3
const D_L: u8 = 0;
const D_R: u8 = 1;
const D_U: u8 = 2;
const D_D: u8 = 3;

const fn opposite(d: u8) -> u8 {
    match d {
        D_L => D_R,
        D_R => D_L,
        D_U => D_D,
        _ => D_U,
    }
}

const fn delta_row(d: u8) -> i32 {
    match d {
        D_U => -1,
        D_D => 1,
        _ => 0,
    }
}

const fn delta_col(d: u8) -> i32 {
    match d {
        D_L => -1,
        D_R => 1,
        _ => 0,
    }
}

// Pipe types.
const P_STRAIGHT: u8 = 0;
const P_CORNER: u8 = 1;
const P_FAT: u8 = 2;

// _CONNS[ptype][rot] = (dir_a, dir_b)
const CONN_STRAIGHT: [(u8, u8); 2] = [(D_L, D_R), (D_U, D_D)];
const CONN_CORNER: [(u8, u8); 4] = [(D_R, D_D), (D_L, D_D), (D_L, D_U), (D_R, D_U)];
const CONN_FAT: [(u8, u8); 2] = [(D_L, D_R), (D_U, D_D)];

fn max_rot(ptype: u8) -> u8 {
    if ptype == P_CORNER {
        4
    } else {
        2
    }
}

fn conn(ptype: u8, rot: u8) -> (u8, u8) {
    match ptype {
        P_STRAIGHT => CONN_STRAIGHT[(rot % 2) as usize],
        P_CORNER => CONN_CORNER[(rot % 4) as usize],
        _ => CONN_FAT[(rot % 2) as usize],
    }
}

fn get_exit(ptype: u8, rot: u8, entry_dir: u8) -> i8 {
    let (a, b) = conn(ptype, rot);
    if a == entry_dir {
        b as i8
    } else if b == entry_dir {
        a as i8
    } else {
        -1
    }
}

fn pipe_for(entry: u8, exit_: u8, rng: &mut u32) -> (u8, u8) {
    // Straight or fat.
    if (entry == D_L && exit_ == D_R) || (entry == D_R && exit_ == D_L) {
        let ptype = if xorshift32(rng) % 5 == 0 { P_FAT } else { P_STRAIGHT };
        return (ptype, 0);
    }
    if (entry == D_U && exit_ == D_D) || (entry == D_D && exit_ == D_U) {
        let ptype = if xorshift32(rng) % 5 == 0 { P_FAT } else { P_STRAIGHT };
        return (ptype, 1);
    }
    // Corner: find which rotation connects these two openings.
    for rot in 0..4u8 {
        let (a, b) = CONN_CORNER[rot as usize];
        if (a == entry && b == exit_) || (b == entry && a == exit_) {
            return (P_CORNER, rot);
        }
    }
    (P_STRAIGHT, 0)
}

struct SolveBuf {
    ptypes: [u8; PLAY_TOTAL],
    rots: [u8; PLAY_TOTAL],
    on_path: [u8; PLAY_TOTAL],
    visited: [u8; PLAY_TOTAL],
}

fn solve(buf: &mut SolveBuf, row: i32, col: i32, entry: u8, rng: &mut u32) -> bool {
    if col == OUTLET_COL as i32 {
        return row == OUTLET_ROW as i32;
    }
    if col < 1 || col > PLAY_COLS as i32 || row < 0 || row >= PLAY_ROWS as i32 {
        return false;
    }
    let pi = (row as usize) * PLAY_COLS + (col as usize - 1);
    if buf.visited[pi] != 0 {
        return false;
    }
    buf.visited[pi] = 1;

    // Build exit candidates [R, U, D] with right-first bias.
    let mut cands = [D_R, D_U, D_D];
    // Fisher-Yates on last two.
    for i in (1..=2usize).rev() {
        let j = (xorshift32(rng) % (i as u32 + 1)) as usize;
        cands.swap(i, j);
    }
    // 70% bias: ensure D_R is first.
    if (xorshift32(rng) % 10) < 7 && cands[0] != D_R {
        let idx = if cands[1] == D_R { 1 } else { 2 };
        cands.swap(0, idx);
    }

    for &exit_dir in &cands {
        if exit_dir == entry {
            continue;
        }
        let nr = row + delta_row(exit_dir);
        let nc = col + delta_col(exit_dir);
        if solve(buf, nr, nc, opposite(exit_dir), rng) {
            let (ptype, rot) = pipe_for(entry, exit_dir, rng);
            buf.ptypes[pi] = ptype;
            buf.rots[pi] = rot;
            buf.on_path[pi] = 1;
            return true;
        }
    }

    buf.visited[pi] = 0;
    false
}

fn gen_solution(buf: &mut SolveBuf, rng: &mut u32) {
    for i in 0..PLAY_TOTAL {
        buf.visited[i] = 0;
    }
    if !solve(buf, OUTLET_ROW as i32, 1, D_L, rng) {
        // Fallback: straight line through outlet row.
        for c in 1..=PLAY_COLS {
            let pi = OUTLET_ROW * PLAY_COLS + (c - 1);
            buf.ptypes[pi] = P_STRAIGHT;
            buf.rots[pi] = 0;
            buf.on_path[pi] = 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Flowing,
    Win,
    Broken,
}

pub struct PipesScene {
    result_popup: Popup,
    session_wins: u32,

    ptypes: [u8; PLAY_TOTAL],
    rots: [u8; PLAY_TOTAL],
    cell_filled: [u8; FILL_TOTAL],

    cur_row: usize,
    cur_col: usize,

    state: State,
    flow_row: i32,
    flow_col: i32,
    flow_entry: u8,
    flow_exit: u8,
    flow_progress: f32,
    flow_speed: f32,
    base_flow_speed: f32,
    inlet_rise_speed: f32,
    start_timer: f32,
    end_timer: f32,
    inlet_rise_px: f32,
    speed_mult: f32,

    options_menu: Menu<PipesAction>,
    menu_active: bool,
}

impl PipesScene {
    pub fn new() -> Self {
        Self {
            result_popup: Popup::new(10, 4, 108, 46),
            options_menu: Menu::new(OPTIONS_ITEMS),
            session_wins: 0,
            ptypes: [0; PLAY_TOTAL],
            rots: [0; PLAY_TOTAL],
            cell_filled: [0; FILL_TOTAL],
            cur_row: OUTLET_ROW,
            cur_col: 0,
            state: State::Flowing,
            flow_row: OUTLET_ROW as i32,
            flow_col: INLET_COL as i32,
            flow_entry: D_L,
            flow_exit: D_R,
            flow_progress: 0.0,
            flow_speed: FLOW_SPEED_NORMAL,
            base_flow_speed: FLOW_SPEED_NORMAL,
            inlet_rise_speed: INLET_RISE_SPEED,
            start_timer: START_DELAY,
            end_timer: 0.0,
            inlet_rise_px: 0.0,
            speed_mult: 1.0,
            menu_active: false,
        }
    }

    fn init_game(&mut self, rng: &mut u32) {
        // Difficulty ramp.
        let min_corners = (2 + self.session_wins).min(10);
        let speed_scale = 1.0 + 0.1 * self.session_wins as f32;
        self.base_flow_speed = (FLOW_SPEED_NORMAL * speed_scale).min(FLOW_SPEED_NORMAL * 3.0);
        self.inlet_rise_speed = (INLET_RISE_SPEED * speed_scale).min(INLET_RISE_SPEED * 3.0);

        let mut buf = SolveBuf {
            ptypes: [0; PLAY_TOTAL],
            rots: [0; PLAY_TOTAL],
            on_path: [0; PLAY_TOTAL],
            visited: [0; PLAY_TOTAL],
        };
        loop {
            gen_solution(&mut buf, rng);
            let mut corner_count: u32 = 0;
            for i in 0..PLAY_TOTAL {
                if buf.on_path[i] != 0 && buf.ptypes[i] == P_CORNER {
                    corner_count += 1;
                }
            }
            if corner_count >= min_corners {
                break;
            }
            for i in 0..PLAY_TOTAL {
                buf.ptypes[i] = 0;
                buf.rots[i] = 0;
                buf.on_path[i] = 0;
            }
        }

        // Fill non-path cells with random pipe types.
        for i in 0..PLAY_TOTAL {
            if buf.on_path[i] == 0 {
                let r = xorshift32(rng) % 10;
                buf.ptypes[i] = if r < 5 {
                    P_STRAIGHT
                } else if r < 9 {
                    P_CORNER
                } else {
                    P_FAT
                };
            }
        }

        // Copy types and scramble rotations. Path cells must differ from solution.
        for i in 0..PLAY_TOTAL {
            self.ptypes[i] = buf.ptypes[i];
            let mr = max_rot(buf.ptypes[i]);
            let mut rot = (xorshift32(rng) % mr as u32) as u8;
            if buf.on_path[i] != 0 && rot == buf.rots[i] {
                rot = (rot + 1) % mr;
            }
            self.rots[i] = rot;
        }

        for i in 0..FILL_TOTAL {
            self.cell_filled[i] = 0;
        }

        self.flow_row = OUTLET_ROW as i32;
        self.flow_col = INLET_COL as i32;
        self.flow_entry = D_L;
        self.flow_exit = D_R;
        self.flow_progress = 0.0;
        self.flow_speed = self.base_flow_speed;
        self.start_timer = START_DELAY;
        self.end_timer = 0.0;
        self.inlet_rise_px = 0.0;
        self.state = State::Flowing;
        self.cur_row = OUTLET_ROW;
        self.cur_col = 0;
    }

    fn advance_flow(&mut self) {
        let fi = self.flow_row as usize * TOTAL_COLS + self.flow_col as usize;
        self.cell_filled[fi] = 1;
        self.flow_progress -= CELL as f32;

        let next_row = self.flow_row + delta_row(self.flow_exit);
        let next_col = self.flow_col + delta_col(self.flow_exit);

        // Outlet column?
        if next_col == OUTLET_COL as i32 {
            if next_row == OUTLET_ROW as i32 {
                self.flow_row = next_row;
                self.flow_col = next_col;
                self.flow_entry = opposite(self.flow_exit);
                self.flow_exit = D_R;
                self.flow_speed = self.base_flow_speed;
            } else {
                self.set_broken();
            }
            return;
        }

        // Out of play bounds?
        if next_row < 0
            || next_row >= PLAY_ROWS as i32
            || next_col < INLET_COL as i32
            || next_col > OUTLET_COL as i32
        {
            self.set_broken();
            return;
        }

        // Play cell (cols 1..=PLAY_COLS).
        if next_col > INLET_COL as i32 && next_col < OUTLET_COL as i32 {
            let pi = next_row as usize * PLAY_COLS + (next_col as usize - 1);
            let ptype = self.ptypes[pi];
            let rot = self.rots[pi];
            let entry_dir = opposite(self.flow_exit);
            let exit_dir = get_exit(ptype, rot, entry_dir);
            if exit_dir < 0 {
                self.set_broken();
                return;
            }
            self.flow_row = next_row;
            self.flow_col = next_col;
            self.flow_entry = entry_dir;
            self.flow_exit = exit_dir as u8;
            let fat_speed = self.base_flow_speed * (FLOW_SPEED_FAT / FLOW_SPEED_NORMAL);
            self.flow_speed = if ptype == P_FAT { fat_speed } else { self.base_flow_speed };
        } else {
            self.set_broken();
        }
    }

    fn set_broken(&mut self) {
        self.state = State::Broken;
        self.end_timer = 0.0;
        self.result_popup.set_text("Burst!\n\nA: New Game", false, true);
    }

    fn set_win(&mut self) {
        let fi = self.flow_row as usize * TOTAL_COLS + self.flow_col as usize;
        self.cell_filled[fi] = 1;
        self.state = State::Win;
        self.end_timer = 0.0;
        self.session_wins += 1;
        let mut text: String<48> = String::new();
        let _ = write!(text, "Connected!\nWins: {}\nA: New Game", self.session_wins);
        self.result_popup.set_text(text.as_str(), false, true);
    }

    fn sprite_for(ptype: u8) -> &'static Sprite {
        match ptype {
            P_STRAIGHT => &PIPE_STRAIGHT,
            P_CORNER => &PIPE_CORNER,
            _ => &PIPE_FAT,
        }
    }

    fn draw_pipes(&self, r: &mut Renderer) {
        let grid_h = PLAY_ROWS as i32 * CELL;
        let grid_right = TOTAL_COLS as i32 * CELL;

        // Border / channel lines (matches the Python framebuffer setup).
        r.draw_line(
            Point::new(GRID_X + 1, GRID_Y),
            Point::new(GRID_X + 1, GRID_Y + grid_h - 1),
        );
        r.draw_line(
            Point::new(GRID_X + 6, GRID_Y),
            Point::new(GRID_X + 6, GRID_Y + 28),
        );
        r.draw_line(
            Point::new(GRID_X + 6, GRID_Y + 36),
            Point::new(GRID_X + 6, GRID_Y + grid_h - 1),
        );
        r.draw_line(
            Point::new(GRID_X + grid_right - 2, GRID_Y),
            Point::new(GRID_X + grid_right - 2, GRID_Y + grid_h - 1),
        );
        r.draw_line(
            Point::new(GRID_X + grid_right - 7, GRID_Y),
            Point::new(GRID_X + grid_right - 7, GRID_Y + 28),
        );
        r.draw_line(
            Point::new(GRID_X + grid_right - 7, GRID_Y + 36),
            Point::new(GRID_X + grid_right - 7, GRID_Y + grid_h - 1),
        );

        let sopts = SpriteOpts { transparent: true, ..Default::default() };

        // Inlet (right-facing).
        let inlet_filled =
            self.cell_filled[OUTLET_ROW * TOTAL_COLS + INLET_COL] as usize;
        r.draw_sprite(
            &PIPE_OUTLET_RIGHT,
            Point::new(
                GRID_X + INLET_COL as i32 * CELL,
                GRID_Y + OUTLET_ROW as i32 * CELL,
            ),
            SpriteOpts { frame: inlet_filled, ..sopts },
        );

        // Outlet (left-facing).
        let outlet_filled =
            self.cell_filled[OUTLET_ROW * TOTAL_COLS + OUTLET_COL] as usize;
        r.draw_sprite(
            &PIPE_OUTLET_LEFT,
            Point::new(
                GRID_X + OUTLET_COL as i32 * CELL,
                GRID_Y + OUTLET_ROW as i32 * CELL,
            ),
            SpriteOpts { frame: outlet_filled, ..sopts },
        );

        // Play cells.
        for row in 0..PLAY_ROWS {
            for pc in 0..PLAY_COLS {
                let grid_col = pc + 1;
                let pi = row * PLAY_COLS + pc;
                let ptype = self.ptypes[pi];
                let rot = self.rots[pi];
                let mr = max_rot(ptype);
                let rot_c = rot % mr;
                let filled = self.cell_filled[row * TOTAL_COLS + grid_col] != 0;
                let base = if ptype == P_CORNER { 4 } else { 2 };
                let frame = if filled {
                    rot_c as usize + base
                } else {
                    rot_c as usize
                };
                r.draw_sprite(
                    Self::sprite_for(ptype),
                    Point::new(
                        GRID_X + grid_col as i32 * CELL,
                        GRID_Y + row as i32 * CELL,
                    ),
                    SpriteOpts { frame, ..sopts },
                );
            }
        }
    }

    fn draw_water(&self, r: &mut Renderer) {
        if self.state != State::Flowing || self.start_timer > 0.0 {
            return;
        }
        let row = self.flow_row;
        let col = self.flow_col;
        let entry = self.flow_entry;
        let exit_ = self.flow_exit;
        let ip = self.flow_progress as i32;
        if ip <= 0 {
            return;
        }

        let cx = GRID_X + col * CELL;
        let cy = GRID_Y + row * CELL;

        // Straight horizontal.
        if (entry == D_L && exit_ == D_R) || (entry == D_R && exit_ == D_L) {
            if entry == D_L {
                r.draw_rect(Point::new(cx, cy + 4), Size::new(ip as u32, 1), true);
            } else {
                r.draw_rect(
                    Point::new(cx + CELL - ip, cy + 4),
                    Size::new(ip as u32, 1),
                    true,
                );
            }
            return;
        }
        // Straight vertical.
        if (entry == D_U && exit_ == D_D) || (entry == D_D && exit_ == D_U) {
            if entry == D_U {
                r.draw_rect(Point::new(cx + 4, cy), Size::new(1, ip as u32), true);
            } else {
                r.draw_rect(
                    Point::new(cx + 4, cy + CELL - ip),
                    Size::new(1, ip as u32),
                    true,
                );
            }
            return;
        }

        // Corner: entry leg 0..=5, exit leg 0..=5.
        let p1 = if ip < 5 { ip } else { 5 };
        let p2 = if ip > 4 { ip - 4 } else { 0 };

        // Entry leg.
        match entry {
            D_L => r.draw_rect(Point::new(cx, cy + 4), Size::new(p1 as u32, 1), true),
            D_R => r.draw_rect(
                Point::new(cx + CELL - p1, cy + 4),
                Size::new(p1 as u32, 1),
                true,
            ),
            D_U => r.draw_rect(Point::new(cx + 4, cy), Size::new(1, p1 as u32), true),
            _ => r.draw_rect(
                Point::new(cx + 4, cy + CELL - p1),
                Size::new(1, p1 as u32),
                true,
            ),
        }

        // Exit leg.
        if p2 > 0 {
            match exit_ {
                D_R => r.draw_rect(Point::new(cx + 4, cy + 4), Size::new(p2 as u32, 1), true),
                D_L => r.draw_rect(
                    Point::new(cx + 5 - p2, cy + 4),
                    Size::new(p2 as u32, 1),
                    true,
                ),
                D_D => r.draw_rect(Point::new(cx + 4, cy + 4), Size::new(1, p2 as u32), true),
                _ => r.draw_rect(
                    Point::new(cx + 4, cy + 5 - p2),
                    Size::new(1, p2 as u32),
                    true,
                ),
            }
        }
    }

    fn draw_cursor(&self, r: &mut Renderer) {
        if self.state != State::Flowing {
            return;
        }
        let cx = GRID_X + (self.cur_col as i32 + 1) * CELL;
        let cy = GRID_Y + self.cur_row as i32 * CELL;
        r.draw_rect(Point::new(cx, cy), Size::new(CELL as u32, CELL as u32), false);
    }

    fn draw_inlet_rise(&self, r: &mut Renderer) {
        let mut rise_px = self.inlet_rise_px as i32;
        if rise_px <= 0 {
            return;
        }
        let grid_h = PLAY_ROWS as i32 * CELL;
        if rise_px > grid_h {
            rise_px = grid_h;
        }
        let top_y = GRID_Y + grid_h - rise_px;
        r.draw_rect(
            Point::new(GRID_X + 2, top_y),
            Size::new(4, rise_px as u32),
            true,
        );
    }

}

impl Scene for PipesScene {
    fn enter(&mut self, ctx: &mut GameContext) {
        self.session_wins = 0;
        let mut rng = ctx.rng;
        self.init_game(&mut rng);
        ctx.rng = rng;
    }

    fn exit(&mut self, ctx: &mut GameContext) {
        if self.session_wins == 0 {
            return;
        }
        let scale = micromath::F32Ext::sqrt(self.session_wins as f32 / 3.0);
        ctx.apply_stat_changes(&[
            (StatId::Intelligence, 3.0 * scale),
            (StatId::Focus,        4.0 * scale),
            (StatId::Loyalty,      0.5 * scale),
        ]);
        let coins = 2 * self.session_wins as i32;
        if coins > 0 {
            ctx.coins += coins;
        }
    }

    fn update(
        &mut self,
        ctx: &mut GameContext,
        buttons: &mut Buttons,
        dt: f32,
    ) -> Option<SceneId> {
        if buttons.was_just_pressed(Button::Menu1) {
            return Some(SceneId::Menu);
        }

        if self.menu_active {
            match self.options_menu.handle_input(buttons, false) {
                MenuResult::Continue => {}
                MenuResult::Closed => {
                    self.menu_active = false;
                }
                MenuResult::Action(PipesAction::NewBoard) => {
                    self.menu_active = false;
                    let mut rng = ctx.rng;
                    self.init_game(&mut rng);
                    ctx.rng = rng;
                }
            }
            return None;
        }

        if self.state == State::Win || self.state == State::Broken {
            self.end_timer += dt;
            if buttons.was_just_pressed(Button::A) {
                let mut rng = ctx.rng;
                self.init_game(&mut rng);
                ctx.rng = rng;
                return None;
            }
            let delay = if self.state == State::Win { WIN_DELAY } else { BROKEN_DELAY };
            if self.end_timer >= delay {
                let mut rng = ctx.rng;
                self.init_game(&mut rng);
                ctx.rng = rng;
            }
            return None;
        }

        if buttons.was_just_pressed(Button::Menu2) {
            self.menu_active = true;
            self.options_menu.reset_to(OPTIONS_ITEMS);
            return None;
        }

        // Cursor movement.
        if buttons.was_just_pressed(Button::Up) && self.cur_row > 0 {
            self.cur_row -= 1;
        } else if buttons.was_just_pressed(Button::Down)
            && self.cur_row < PLAY_ROWS - 1
        {
            self.cur_row += 1;
        } else if buttons.was_just_pressed(Button::Left) && self.cur_col > 0 {
            self.cur_col -= 1;
        } else if buttons.was_just_pressed(Button::Right)
            && self.cur_col < PLAY_COLS - 1
        {
            self.cur_col += 1;
        }

        // Rotate with A (only if not yet filled and not the current flow cell).
        if buttons.was_just_pressed(Button::A) {
            let grid_col = self.cur_col + 1;
            let fi = self.cur_row * TOTAL_COLS + grid_col;
            let is_flow_cell = self.flow_row == self.cur_row as i32
                && self.flow_col == grid_col as i32;
            if self.cell_filled[fi] == 0 && !is_flow_cell {
                let pi = self.cur_row * PLAY_COLS + self.cur_col;
                let mr = max_rot(self.ptypes[pi]);
                self.rots[pi] = (self.rots[pi] + 1) % mr;
            }
        }

        self.speed_mult = if buttons.is_pressed(Button::B) { 7.0 } else { 1.0 };

        // Setup window: water rises in the inlet channel.
        let grid_h = PLAY_ROWS as i32 * CELL;
        if self.inlet_rise_px < grid_h as f32 {
            self.inlet_rise_px = (self.inlet_rise_px
                + self.inlet_rise_speed * dt * self.speed_mult)
                .min(grid_h as f32);
        }

        if self.start_timer > 0.0 {
            // Flow starts once the water rises to the centerline of the outlet row.
            let outlet_threshold =
                grid_h as f32 - (OUTLET_ROW as f32 * CELL as f32 + CELL as f32 / 2.0);
            if self.inlet_rise_px >= outlet_threshold {
                self.start_timer = 0.0;
            }
            return None;
        }

        self.flow_progress += self.flow_speed * dt * self.speed_mult;

        while self.flow_progress >= CELL as f32 {
            if self.flow_col == OUTLET_COL as i32 {
                self.set_win();
                break;
            }
            self.advance_flow();
            if self.state != State::Flowing {
                break;
            }
        }

        None
    }

    fn draw(&self, _ctx: &GameContext, renderer: &mut Renderer, _dt_ms: u64) {
        if self.menu_active {
            self.options_menu.draw(renderer);
            return;
        }

        self.draw_pipes(renderer);
        self.draw_water(renderer);
        self.draw_cursor(renderer);

        if self.state == State::Win || self.state == State::Broken {
            self.result_popup.draw(renderer, false);
        }

        self.draw_inlet_rise(renderer);
    }
}
