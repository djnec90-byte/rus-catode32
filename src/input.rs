use esp_hal::{
    gpio::Input,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Button {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    A = 4,
    B = 5,
    Menu1 = 6,
    Menu2 = 7,
}

impl Button {
    pub const ALL: [Button; 8] = [
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::A,
        Button::B,
        Button::Menu1,
        Button::Menu2,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Button::Up => "U",
            Button::Down => "D",
            Button::Left => "L",
            Button::Right => "R",
            Button::A => "A",
            Button::B => "B",
            Button::Menu1 => "M1",
            Button::Menu2 => "M2",
        }
    }
}

const DEBOUNCE: Duration = Duration::from_millis(50);

pub struct Buttons {
    pins: [Input<'static>; 8],
    edge_state: [bool; 8],
    last_press: [Option<Instant>; 8],
}

impl Buttons {
    pub fn new(pins: [Input<'static>; 8]) -> Self {
        Self {
            pins,
            edge_state: [false; 8],
            last_press: [None; 8],
        }
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        self.pins[button as usize].is_low()
    }

    pub fn was_just_pressed(&mut self, button: Button) -> bool {
        let idx = button as usize;
        let pressed = self.pins[idx].is_low();
        let was = self.edge_state[idx];
        let debounced = self.last_press[idx].map_or(true, |t| t.elapsed() > DEBOUNCE);

        if pressed && !was && debounced {
            self.edge_state[idx] = true;
            self.last_press[idx] = Some(Instant::now());
            return true;
        }
        if !pressed && was {
            self.edge_state[idx] = false;
        }
        false
    }

    pub fn pressed_mask(&self) -> u8 {
        let mut mask = 0u8;
        for i in 0..8 {
            if self.pins[i].is_low() {
                mask |= 1 << i;
            }
        }
        mask
    }

    pub fn any_pressed(&self) -> bool {
        self.pressed_mask() != 0
    }

    /// Mark every currently-held button as already seen, so the next
    /// `was_just_pressed()` call will not report it as a fresh press.
    /// Called by the sleep manager on wake so the button that triggered
    /// the wake is not also passed through as a game action.
    pub fn consume_all(&mut self) {
        let now = Instant::now();
        for i in 0..8 {
            self.edge_state[i] = self.pins[i].is_low();
            self.last_press[i] = Some(now);
        }
    }

    pub fn direction(&self) -> (i8, i8) {
        let mut dx = 0;
        let mut dy = 0;
        if self.is_pressed(Button::Up) { dy -= 1; }
        if self.is_pressed(Button::Down) { dy += 1; }
        if self.is_pressed(Button::Left) { dx -= 1; }
        if self.is_pressed(Button::Right) { dx += 1; }
        (dx, dy)
    }
}
