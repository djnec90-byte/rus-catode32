use esp_hal::{
    gpio::{Input, InputConfig, Pull},
    i2c::master::{Config, I2c},
    interrupt::software::SoftwareInterrupt,
    peripherals::{Peripherals, FLASH, TIMG0, WIFI},
    rng::Rng,
    time::Rate,
    Blocking,
};

use crate::{input::Buttons, led::Led};

pub const DISPLAY_WIDTH: u16 = 128;
pub const DISPLAY_HEIGHT: u16 = 64;
pub const I2C_FREQ_KHZ: u32 = 400;

pub struct Board {
    pub i2c: I2c<'static, Blocking>,
    pub buttons: Buttons,
    pub rng: Rng,
    pub led: Led,
    pub flash: FLASH<'static>,
    pub timg0: TIMG0<'static>,
    pub sw_int0: SoftwareInterrupt<'static, 0>,
    pub wifi: WIFI<'static>,
}

pub fn init(peripherals: Peripherals) -> Board {
    let i2c_config = Config::default().with_frequency(Rate::from_khz(I2C_FREQ_KHZ));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .unwrap()
        .with_sda(peripherals.GPIO4)
        .with_scl(peripherals.GPIO7);

    let input_config = InputConfig::default().with_pull(Pull::Up);
    let buttons = Buttons::new([
        Input::new(peripherals.GPIO14, input_config),
        Input::new(peripherals.GPIO18, input_config),
        Input::new(peripherals.GPIO20, input_config),
        Input::new(peripherals.GPIO19, input_config),
        Input::new(peripherals.GPIO1, input_config),
        Input::new(peripherals.GPIO0, input_config),
        Input::new(peripherals.GPIO3, input_config),
        Input::new(peripherals.GPIO2, input_config),
    ]);

    let led = Led::new(peripherals.RMT, peripherals.GPIO8);

    let sw = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    Board {
        i2c,
        buttons,
        rng: Rng::new(),
        led,
        flash: peripherals.FLASH,
        timg0: peripherals.TIMG0,
        sw_int0: sw.software_interrupt0,
        wifi: peripherals.WIFI,
    }
}
