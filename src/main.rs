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
mod game;
mod gardening_ui;
mod input;
mod led;
mod location_scene;
mod pet_names;
mod pet_seed;
mod plant_renderer;
mod plant_system;
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

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, main};
use esp_println::println;

use crate::{game::Game, render::Renderer};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let board = board::init(peripherals);
    storage::init(board.flash);
    let renderer = Renderer::new(board.i2c);

    println!("catode32 v0.10.0 — behavior framework");

    Game::new(renderer, board.buttons, board.rng, board.led).run();
}
