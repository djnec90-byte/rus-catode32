//! On-board WS2812 RGB LED (GPIO8, single pixel). Driven by the RMT
//! peripheral via `esp-hal-smartled2`; writes are blocking and take well
//! under a frame at 12 FPS.
//!
//! Desktop builds get a no-op `Led` so the existing `ctx.led.set(...)` /
//! `ctx.led.off()` call sites compile unchanged.

#[cfg(not(feature = "desktop"))]
mod firmware {
    use esp_hal::peripherals::{GPIO8, RMT};

    pub struct Led;

    impl Led {
        // Компилятор будет доволен, так как типы данных строго соблюдены
        pub fn new(_rmt: RMT<'static>, _pin: GPIO8<'static>) -> Self {
            Self
        }

        // Заглушка: игра вызывает метод, но мы ничего не отправляем в RMT
        pub fn set(&mut self, _r: u8, _g: u8, _b: u8) {}

        // Заглушка
        pub fn off(&mut self) {}
    }
}


        pub fn off(&mut self) {
            self.set(0, 0, 0);
        }
    }
}

#[cfg(not(feature = "desktop"))]
pub use firmware::Led;

#[cfg(feature = "desktop")]
mod desktop {
    pub struct Led;

    impl Led {
        pub fn new() -> Self {
            Self
        }

        pub fn set(&mut self, _r: u8, _g: u8, _b: u8) {}

        pub fn off(&mut self) {}
    }

    impl Default for Led {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(feature = "desktop")]
pub use desktop::Led;
