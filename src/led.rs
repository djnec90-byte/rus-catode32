use esp_hal::{
    Blocking,
    peripherals::{GPIO8, RMT},
    rmt::Rmt,
    time::Rate,
};
use esp_hal_smartled::{buffer_size, Ws2812SmartLeds};
use smart_leds::{SmartLedsWrite, RGB8};

const LED_BUFFER: usize = buffer_size::<RGB8>(1);

type Strip = Ws2812SmartLeds<'static, LED_BUFFER, Blocking>;

/// On-board WS2812 RGB LED (GPIO8, single pixel). Driven by the RMT
/// peripheral via `esp-hal-smartled2`; writes are blocking and take well
/// under a frame at 12 FPS.
pub struct Led {
    strip: Strip,
}

impl Led {
    pub fn new(rmt: RMT<'static>, pin: GPIO8<'static>) -> Self {
        let rmt = Rmt::new(rmt, Rate::from_mhz(80)).unwrap();
        let strip = Ws2812SmartLeds::new(rmt.channel0, pin).unwrap();
        Self { strip }
    }

    pub fn set(&mut self, r: u8, g: u8, b: u8) {
        let _ = self.strip.write(core::iter::once(RGB8 { r, g, b }));
    }

    pub fn off(&mut self) {
        self.set(0, 0, 0);
    }
}
