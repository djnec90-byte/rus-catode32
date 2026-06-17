#![no_std]
#![no_main]

mod assets;
mod behavior;
mod behaviors;
mod board;
mod character;
mod context;
mod game;
mod input;
mod render;
mod scene;
mod scenes;

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
    let renderer = Renderer::new(board.i2c);

    println!("catode32 v0.10.0 — behavior framework");

    Game::new(renderer, board.buttons).run();
}
