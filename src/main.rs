#![no_std]
#![no_main]

mod assets;
mod behavior;
mod behaviors;
mod board;
mod character;
mod clock;
mod context;
mod entities;
mod environment;
mod espnow_manager;
mod espnow_msg;
mod game;
mod gardening_ui;
mod input;
mod led;
mod location_scene;
mod pet_names;
mod pet_seed;
mod plant_renderer;
mod plant_system;
mod radio;
mod rand;
mod render;
mod save;
mod scene;
mod scenes;
mod sky;
mod sleep_manager;
mod storage;
mod temperature_system;
mod time_system;
mod transition;
mod ui;
mod weather_system;
mod wifi_tracker;

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, main, timer::timg::TimerGroup};
use esp_println::println;

use crate::{game::Game, render::Renderer};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Heap for the wifi stack. esp-radio (the wifi driver) and esp-rtos (the
    // scheduler it needs) both allocate from a global heap. 64 KB is the size
    // the esp-rs maintainers recommend for scan-only wifi; it holds the
    // driver's permanent buffers plus transient per-scan AP lists with
    // comfortable headroom. Nothing else in this firmware allocates.
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let board = board::init(peripherals);
    storage::init(board.flash);
    let renderer = Renderer::new(board.i2c);

    println!("catode32 v0.10.0 — behavior framework");

    // Start the preemptive scheduler. The radio stack drives its event loop
    // on a task scheduled by esp-rtos, so the scheduler must be running
    // before `esp_radio::wifi::new`. Our main game loop runs on top of this
    // as the "main" task.
    let timg0 = TimerGroup::new(board.timg0);
    esp_rtos::start(timg0.timer0, board.sw_int0);

    // No wifi init at boot — the radio is brought up on demand by
    // `radio::acquire` and dropped again on the last release. The
    // peripheral and an empty ESP-NOW manager are stashed on the game
    // context so the first acquire can do the actual `wifi::new`.
    Game::new(
        renderer,
        board.buttons,
        board.rng,
        board.led,
        board.wifi,
        espnow_manager::EspNowManager::new(),
    )
    .run();
}
