use esp_hal::{
    i2c::master::{Config, I2c},
    peripherals::Peripherals,
    time::Rate,
    Blocking,
};

pub const DISPLAY_WIDTH: u16 = 128;
pub const DISPLAY_HEIGHT: u16 = 64;
pub const I2C_FREQ_KHZ: u32 = 400;

pub struct Board {
    pub i2c: I2c<'static, Blocking>,
}

pub fn init(peripherals: Peripherals) -> Board {
    let config = Config::default().with_frequency(Rate::from_khz(I2C_FREQ_KHZ));
    let i2c = I2c::new(peripherals.I2C0, config)
        .unwrap()
        .with_sda(peripherals.GPIO4)
        .with_scl(peripherals.GPIO7);
    Board { i2c }
}
