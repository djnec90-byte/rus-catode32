//! Platformer level data and parser.
//!
//! Character key:
//!   '1' / '2'  solid terrain block (variants 0 / 1)
//!   '_'        one-way platform (consecutive `_` on the same row coalesce)
//!   'g'        grass decoration (sprite picked randomly at parse time)
//!   ','        background tile (group 0; variant picked randomly)
//!   'S'        slime enemy spawn point
//!   '$'        player spawn point
//!   '@'        checkpoint
//!   '#'        level exit door (paired with `-` dest lines at file bottom)
//!   'X'        locked door (also paired with a `- dest` line)
//!   'K'        key pickup
//!   'o'        coin pickup
//!   any other  empty
//!
//! Door destinations: after the grid, lines starting with `-` name the
//! destination level for each `#` / `X` door in reading order.

use heapless::{String, Vec};

use crate::assets::platformer_terrain::{
    TILE_BOTTOM, TILE_BOTTOM_LEFT, TILE_BOTTOM_RIGHT, TILE_LEFT_RIGHT, TILE_LEFT_RIGHT_BOTTOM,
    TILE_SIDE_LEFT, TILE_SIDE_RIGHT, TILE_TOP, TILE_TOP_BOTTOM, TILE_TOP_LEFT,
    TILE_TOP_LEFT_BOTTOM, TILE_TOP_LEFT_BOTTOM_RIGHT, TILE_TOP_LEFT_RIGHT, TILE_TOP_RIGHT,
    TILE_TOP_RIGHT_BOTTOM,
};
use crate::rand::xorshift32;

pub const BLOCK_W: u16 = 8;
pub const BLOCK_H: u16 = 8;
pub const CHUNK_W: u16 = 128;
pub const CHUNK_H: u16 = 64;

pub const GRASS_VARIANTS: u8 = 5;
pub const BG_GROUP_COMMA: u8 = 0;
pub const BG_GROUP_COMMA_VARIANTS: u8 = 2;

// Heapless capacities sized for the worst case across all six authored levels
// (see survey in the platformer port plan). All numbers have a small headroom.
pub const MAX_BLOCKS: usize = 1280;
pub const MAX_BG: usize = 64;
pub const MAX_GRASS: usize = 64;
pub const MAX_PLATFORMS: usize = 64;
pub const MAX_CHECKPOINTS: usize = 4;
pub const MAX_DOORS: usize = 4;
pub const MAX_LOCKED_DOORS: usize = 4;
pub const MAX_KEYS: usize = 4;
pub const MAX_COINS: usize = 32;
pub const MAX_SLIMES: usize = 32;
pub const DEST_LEN: usize = 16;

// Worst case chunk grid: level 02 is 221 cols × 27 rows = 1768 × 216 px → 14 × 4
// chunks. Round up to powers of two for cleanly hashable chunk indices.
pub const MAX_CHUNK_COLS: u16 = 16;
pub const MAX_CHUNK_ROWS: u16 = 8;
pub const MAX_CHUNKS: usize = (MAX_CHUNK_COLS as usize) * (MAX_CHUNK_ROWS as usize);

#[inline]
pub fn chunk_idx(chunk_col: u16, chunk_row: u16) -> u16 {
    chunk_row * MAX_CHUNK_COLS + chunk_col
}

#[derive(Clone, Copy)]
pub struct Block {
    pub x: u16,
    pub y: u16,
    pub tile_type: u8,
    pub variant: u8,
}

#[derive(Clone, Copy)]
pub struct BgTile {
    pub x: u16,
    pub y: u16,
    pub group: u8,
    pub variant: u8,
}

#[derive(Clone, Copy)]
pub struct GrassTile {
    /// World-x of the centre of the grass tuft.
    pub cx: u16,
    /// Surface y (bottom of the grass sprite when drawn).
    pub surface_y: u16,
    pub variant: u8,
}

#[derive(Clone, Copy)]
pub struct Platform {
    pub x: u16,
    pub y: u16,
    pub w: u16,
}

#[derive(Clone, Copy)]
pub struct Checkpoint {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone)]
pub struct DoorSpec {
    pub x: u16,
    pub y: u16,
    pub dest: String<DEST_LEN>,
}

#[derive(Clone, Copy)]
pub struct ItemSpawn {
    pub x: u16,
    pub y: u16,
}

pub struct LevelData {
    pub world_w: u16,
    pub world_h: u16,
    pub spawn: (u16, u16),
    pub blocks: Vec<Block, MAX_BLOCKS>,
    pub bg: Vec<BgTile, MAX_BG>,
    pub grass: Vec<GrassTile, MAX_GRASS>,
    pub platforms: Vec<Platform, MAX_PLATFORMS>,
    pub checkpoints: Vec<Checkpoint, MAX_CHECKPOINTS>,
    pub doors: Vec<DoorSpec, MAX_DOORS>,
    pub locked_doors: Vec<DoorSpec, MAX_LOCKED_DOORS>,
    pub keys: Vec<ItemSpawn, MAX_KEYS>,
    pub coins: Vec<ItemSpawn, MAX_COINS>,
    pub slime_spawns: Vec<ItemSpawn, MAX_SLIMES>,
    /// Per-chunk index into `blocks`. `block_chunks[chunk_idx]` = (start, end);
    /// blocks for the chunk live at `blocks[start..end]`. Empty chunks have
    /// `start == end`. Blocks within `blocks` are sorted by chunk index.
    pub block_chunks: [(u16, u16); MAX_CHUNKS],
    pub bg_chunks: [(u16, u16); MAX_CHUNKS],
    pub grass_chunks: [(u16, u16); MAX_CHUNKS],
}

impl LevelData {
    pub const fn empty() -> Self {
        Self {
            world_w: 0,
            world_h: 0,
            spawn: (8, 8),
            blocks: Vec::new(),
            bg: Vec::new(),
            grass: Vec::new(),
            platforms: Vec::new(),
            checkpoints: Vec::new(),
            doors: Vec::new(),
            locked_doors: Vec::new(),
            keys: Vec::new(),
            coins: Vec::new(),
            slime_spawns: Vec::new(),
            block_chunks: [(0, 0); MAX_CHUNKS],
            bg_chunks: [(0, 0); MAX_CHUNKS],
            grass_chunks: [(0, 0); MAX_CHUNKS],
        }
    }

    pub fn blocks_in_chunk(&self, chunk_col: u16, chunk_row: u16) -> &[Block] {
        if chunk_col >= MAX_CHUNK_COLS || chunk_row >= MAX_CHUNK_ROWS {
            return &[];
        }
        let (s, e) = self.block_chunks[chunk_idx(chunk_col, chunk_row) as usize];
        &self.blocks[s as usize..e as usize]
    }

    pub fn bg_in_chunk(&self, chunk_col: u16, chunk_row: u16) -> &[BgTile] {
        if chunk_col >= MAX_CHUNK_COLS || chunk_row >= MAX_CHUNK_ROWS {
            return &[];
        }
        let (s, e) = self.bg_chunks[chunk_idx(chunk_col, chunk_row) as usize];
        &self.bg[s as usize..e as usize]
    }

    pub fn grass_in_chunk(&self, chunk_col: u16, chunk_row: u16) -> &[GrassTile] {
        if chunk_col >= MAX_CHUNK_COLS || chunk_row >= MAX_CHUNK_ROWS {
            return &[];
        }
        let (s, e) = self.grass_chunks[chunk_idx(chunk_col, chunk_row) as usize];
        &self.grass[s as usize..e as usize]
    }
}

// ── Embedded level text files ────────────────────────────────────────────────

pub static LEVEL_01_TXT: &str = include_str!("levels/level_01.txt");
pub static LEVEL_02_TXT: &str = include_str!("levels/level_02.txt");
pub static LEVEL_03_TXT: &str = include_str!("levels/level_03.txt");
pub static LEVEL_04_TXT: &str = include_str!("levels/level_04.txt");
pub static LEVEL_05_TXT: &str = include_str!("levels/level_05.txt");
pub static LEVEL_06_TXT: &str = include_str!("levels/level_06.txt");

pub fn level_text(name: &str) -> Option<&'static str> {
    match name {
        "level_01" => Some(LEVEL_01_TXT),
        "level_02" => Some(LEVEL_02_TXT),
        "level_03" => Some(LEVEL_03_TXT),
        "level_04" => Some(LEVEL_04_TXT),
        "level_05" => Some(LEVEL_05_TXT),
        "level_06" => Some(LEVEL_06_TXT),
        _ => None,
    }
}

// ── Parser ───────────────────────────────────────────────────────────────────

fn is_terrain_char(c: char) -> bool {
    c == '1' || c == '2'
}

fn is_terrain_at(grid: &[&str], r: i32, c: i32) -> bool {
    if r < 0 || c < 0 || (r as usize) >= grid.len() {
        return false;
    }
    let row = grid[r as usize];
    let bytes = row.as_bytes();
    if (c as usize) >= bytes.len() {
        return false;
    }
    let ch = bytes[c as usize] as char;
    is_terrain_char(ch)
}

/// Return the TILE_* constant for the terrain cell at (r, c), or `None` if the
/// cell is fully interior (all four neighbours are also terrain) and should be
/// skipped — no sprite needs to be emitted for it.
fn tile_type(grid: &[&str], r: i32, c: i32) -> Option<u8> {
    let above = is_terrain_at(grid, r - 1, c);
    let below = is_terrain_at(grid, r + 1, c);
    let left = is_terrain_at(grid, r, c - 1);
    let right = is_terrain_at(grid, r, c + 1);

    let top_exp = !above;
    let bot_exp = !below;

    if !top_exp && !bot_exp {
        if !left && !right {
            return Some(TILE_LEFT_RIGHT);
        }
        if !left {
            return Some(TILE_SIDE_LEFT);
        }
        if !right {
            return Some(TILE_SIDE_RIGHT);
        }
        return None;
    }

    if top_exp && !bot_exp {
        if !left && !right {
            return Some(TILE_TOP_LEFT_RIGHT);
        }
        if !left {
            return Some(TILE_TOP_LEFT);
        }
        if !right {
            return Some(TILE_TOP_RIGHT);
        }
        return Some(TILE_TOP);
    }

    if !top_exp && bot_exp {
        if !left && !right {
            return Some(TILE_LEFT_RIGHT_BOTTOM);
        }
        if !left {
            return Some(TILE_BOTTOM_LEFT);
        }
        if !right {
            return Some(TILE_BOTTOM_RIGHT);
        }
        return Some(TILE_BOTTOM);
    }

    // single-height (top and bottom both exposed)
    if !left && !right {
        return Some(TILE_TOP_LEFT_BOTTOM_RIGHT);
    }
    if !left {
        return Some(TILE_TOP_LEFT_BOTTOM);
    }
    if !right {
        return Some(TILE_TOP_RIGHT_BOTTOM);
    }
    Some(TILE_TOP_BOTTOM)
}

pub fn parse_level(text: &str, rng: &mut u32) -> LevelData {
    let mut data = LevelData::empty();

    // Split into grid lines and destination lines.
    // Door-destination lines start with '-' and live at the end of the file,
    // optionally separated from the grid by blank lines.
    let mut all_lines: Vec<&str, 64> = Vec::new();
    for line in text.lines() {
        let _ = all_lines.push(line);
    }

    // Peel destination lines off the bottom.
    let mut dest_lines: Vec<&str, 8> = Vec::new();
    while let Some(last) = all_lines.last() {
        let trimmed = last.trim();
        if trimmed.starts_with('-') {
            let _ = dest_lines.insert(0, trimmed[1..].trim());
            all_lines.pop();
        } else {
            break;
        }
    }
    // Drop any blank separator lines between the grid and destinations.
    while let Some(last) = all_lines.last() {
        if last.trim().is_empty() {
            all_lines.pop();
        } else {
            break;
        }
    }

    if all_lines.is_empty() {
        return data;
    }

    let grid: &[&str] = &all_lines;
    let num_rows = grid.len();
    let num_cols = grid.iter().map(|l| l.len()).max().unwrap_or(0);

    data.world_w = (num_cols as u16) * BLOCK_W;
    data.world_h = (num_rows as u16) * BLOCK_H;

    // Reading-order list of doors paired with `dest_lines`.
    // (x, y, is_locked)
    let mut all_door_positions: Vec<(u16, u16, bool), 8> = Vec::new();

    for r in 0..num_rows {
        let row = grid[r].as_bytes();
        let row_len = row.len();
        let mut c = 0usize;
        while c < num_cols {
            let ch = if c < row_len { row[c] as char } else { '.' };

            if ch == '1' || ch == '2' {
                let variant: u8 = if ch == '1' { 0 } else { 1 };
                if let Some(tt) = tile_type(grid, r as i32, c as i32) {
                    let bx = (c as u16) * BLOCK_W;
                    let by = (r as u16) * BLOCK_H;
                    let _ = data.blocks.push(Block {
                        x: bx,
                        y: by,
                        tile_type: tt,
                        variant,
                    });
                }
                c += 1;
                continue;
            }

            if ch == '_' {
                // Consume the full run as one platform.
                let start_c = c;
                while c < num_cols && c < row_len && row[c] == b'_' {
                    c += 1;
                }
                let px = (start_c as u16) * BLOCK_W;
                let py = (r as u16) * BLOCK_H;
                let pw = ((c - start_c) as u16) * BLOCK_W;
                let _ = data.platforms.push(Platform {
                    x: px,
                    y: py,
                    w: pw,
                });
                continue;
            }

            match ch {
                'g' => {
                    let cx = (c as u16) * BLOCK_W + BLOCK_W / 2;
                    let sy = ((r + 1) as u16) * BLOCK_H;
                    let v = (xorshift32(rng) % GRASS_VARIANTS as u32) as u8;
                    let _ = data.grass.push(GrassTile {
                        cx,
                        surface_y: sy,
                        variant: v,
                    });
                }
                ',' => {
                    let bx = (c as u16) * BLOCK_W;
                    let by = (r as u16) * BLOCK_H;
                    let v = (xorshift32(rng) % BG_GROUP_COMMA_VARIANTS as u32) as u8;
                    let _ = data.bg.push(BgTile {
                        x: bx,
                        y: by,
                        group: BG_GROUP_COMMA,
                        variant: v,
                    });
                }
                'S' => {
                    let x = (c as u16) * BLOCK_W + BLOCK_W / 2;
                    let fy = ((r + 1) as u16) * BLOCK_H;
                    let _ = data.slime_spawns.push(ItemSpawn { x, y: fy });
                }
                '$' => {
                    let x = (c as u16) * BLOCK_W + BLOCK_W / 2;
                    let fy = ((r + 1) as u16) * BLOCK_H;
                    data.spawn = (x, fy);
                }
                '@' => {
                    let x = (c as u16) * BLOCK_W;
                    let y = ((r + 1) as u16) * BLOCK_H;
                    let _ = data.checkpoints.push(Checkpoint { x, y });
                }
                '#' => {
                    let x = (c as u16) * BLOCK_W;
                    let y = ((r + 1) as u16) * BLOCK_H;
                    let _ = all_door_positions.push((x, y, false));
                }
                'X' => {
                    let x = (c as u16) * BLOCK_W;
                    let y = ((r + 1) as u16) * BLOCK_H;
                    let _ = all_door_positions.push((x, y, true));
                }
                'K' => {
                    let x = (c as u16) * BLOCK_W + BLOCK_W / 2;
                    let y = ((r + 1) as u16) * BLOCK_H;
                    let _ = data.keys.push(ItemSpawn { x, y });
                }
                'o' => {
                    let x = (c as u16) * BLOCK_W + BLOCK_W / 2;
                    let y = ((r + 1) as u16) * BLOCK_H;
                    let _ = data.coins.push(ItemSpawn { x, y });
                }
                _ => {}
            }
            c += 1;
        }
    }

    // Pair doors with destinations in reading order.
    for (i, &(x, y, is_locked)) in all_door_positions.iter().enumerate() {
        let dest_str = dest_lines.get(i).copied().unwrap_or("");
        let mut dest: String<DEST_LEN> = String::new();
        let _ = dest.push_str(dest_str);
        if is_locked {
            let _ = data.locked_doors.push(DoorSpec { x, y, dest });
        } else {
            let _ = data.doors.push(DoorSpec { x, y, dest });
        }
    }

    // Sort the chunk-keyed arrays so each chunk's entries are contiguous, then
    // build the (start, end) lookup tables. Sort by (chunk_idx, y, x) so that
    // *within* a chunk blocks are iterated row-major (topmost first, then
    // left-to-right). Physics relies on this order: descending collision picks
    // the first y-match, and horizontal collision picks the leftmost x-match.
    // `sort_unstable` doesn't preserve insertion order on ties — without the
    // y/x tiebreakers the cat phases through stacked ledges and shoots right
    // into multi-block walls.
    data.blocks.sort_unstable_by_key(|b| {
        (chunk_idx(b.x / CHUNK_W, b.y / CHUNK_H), b.y, b.x)
    });
    data.bg
        .sort_unstable_by_key(|t| chunk_idx(t.x / CHUNK_W, t.y / CHUNK_H));
    data.grass
        .sort_unstable_by_key(|g| chunk_idx(g.cx / CHUNK_W, g.surface_y / CHUNK_H));

    build_chunk_index(&data.blocks, |b| chunk_idx(b.x / CHUNK_W, b.y / CHUNK_H), &mut data.block_chunks);
    build_chunk_index(&data.bg, |t| chunk_idx(t.x / CHUNK_W, t.y / CHUNK_H), &mut data.bg_chunks);
    build_chunk_index(&data.grass, |g| chunk_idx(g.cx / CHUNK_W, g.surface_y / CHUNK_H), &mut data.grass_chunks);

    data
}

fn build_chunk_index<T, F>(items: &[T], key_of: F, out: &mut [(u16, u16); MAX_CHUNKS])
where
    F: Fn(&T) -> u16,
{
    for slot in out.iter_mut() {
        *slot = (0, 0);
    }
    if items.is_empty() {
        return;
    }
    // Items are sorted by chunk_idx (caller responsibility). Walk once and
    // record each chunk's [start, end) range.
    let mut i = 0usize;
    while i < items.len() {
        let k = key_of(&items[i]);
        let start = i;
        while i < items.len() && key_of(&items[i]) == k {
            i += 1;
        }
        if (k as usize) < out.len() {
            out[k as usize] = (start as u16, i as u16);
        }
    }
}
