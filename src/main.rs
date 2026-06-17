#![no_std]
#![no_main]

mod board;

use core::fmt::Write;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    main,
    time::{Duration, Instant},
};
use esp_println::println;
use heapless::String;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let board = board::init(peripherals);

    let interface = I2CDisplayInterface::new(board.i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    println!("catode32 v0.10.0 — display up");

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let border_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    let mut counter: u32 = 0;
    loop {
        display.clear_buffer();

        Rectangle::new(
            Point::new(0, 0),
            Size::new(board::DISPLAY_WIDTH as u32, board::DISPLAY_HEIGHT as u32),
        )
        .into_styled(border_style)
        .draw(&mut display)
        .unwrap();

        Text::with_baseline("catode32 v0.10", Point::new(4, 4), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        let mut buf: String<32> = String::new();
        write!(buf, "tick: {}", counter).unwrap();
        Text::with_baseline(buf.as_str(), Point::new(4, 20), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        display.flush().unwrap();

        println!("tick {}", counter);
        counter += 1;

        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(1000) {}
    }
}
