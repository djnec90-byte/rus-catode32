//! Hardware RNG handle. Firmware re-exports `esp_hal::rng::Rng` (the
//! peripheral); desktop provides a `getrandom`-free xorshift64 backed by a
//! one-time epoch seed, which is plenty for picking pet seeds and shuffling
//! menus on a simulator.

#[cfg(not(feature = "desktop"))]
pub use esp_hal::rng::Rng;

#[cfg(feature = "desktop")]
mod desktop {
    use std::cell::Cell;

    fn seed_from_clock() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15)
            | 1
    }

    /// xorshift64* — adequate for game decisions on the desktop simulator.
    pub struct Rng(Cell<u64>);

    impl Rng {
        pub fn new() -> Self {
            Self(Cell::new(seed_from_clock()))
        }

        pub fn random(&self) -> u32 {
            self.next_u64() as u32
        }

        pub fn read(&self, buf: &mut [u8]) {
            for chunk in buf.chunks_mut(8) {
                let v = self.next_u64();
                let bytes = v.to_le_bytes();
                let n = chunk.len();
                chunk.copy_from_slice(&bytes[..n]);
            }
        }

        fn next_u64(&self) -> u64 {
            let mut x = self.0.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0.set(x);
            x.wrapping_mul(0x2545F4914F6CDD1D)
        }
    }

    impl Default for Rng {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(feature = "desktop")]
pub use desktop::Rng;
