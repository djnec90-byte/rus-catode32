pub fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    if x == 0 {
        x = 1;
    }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

pub fn rand_f32(state: &mut u32) -> f32 {
    xorshift32(state) as f32 / u32::MAX as f32
}

pub fn rand_range_f32(state: &mut u32, lo: f32, hi: f32) -> f32 {
    lo + rand_f32(state) * (hi - lo)
}

pub fn rand_range_u32(state: &mut u32, lo: u32, hi_inclusive: u32) -> u32 {
    lo + xorshift32(state) % (hi_inclusive - lo + 1)
}

pub fn rand_bool(state: &mut u32, threshold_0_to_1: f32) -> bool {
    rand_f32(state) < threshold_0_to_1
}
