//! Hardware Abstraction Layer for the LilyGo T-Display-S3
//! 
//! Converts the low level hardware interfaces (GPIO, I2C) into higher level abstractions (Display, Button, I2C)
//!  that can be used in the main application logic without worrying about the underlying hardware details.
use button_driver::{ButtonConfig, InstantProvider};
use esp_hal::{
    delay::Delay,
    gpio::{InputConfig, OutputConfig},
    i2c::master::I2c,
    time::{Instant, Rate},
};
use mipidsi::{
    interface::{Generic8BitBus, ParallelInterface},
    models::ST7789,
    options::ColorInversion,
    Builder,
};

// Some type aliases to make the code more readable outside the hal module
pub type PinOut = esp_hal::gpio::Output<'static>;
pub type Display = mipidsi::Display<
    mipidsi::interface::ParallelInterface<
        mipidsi::interface::Generic8BitBus<
            PinOut,
            PinOut,
            PinOut,
            PinOut,
            PinOut,
            PinOut,
            PinOut,
            PinOut,
        >,
        PinOut,
        PinOut,
    >,
    mipidsi::models::ST7789,
    PinOut,
>;
pub type Button = button_driver::Button<GPIODriver, WrappedInstant>;

pub mod lilygo_display_config {
    use mipidsi::options::{Orientation, Rotation};
    /// from datasheet
    pub const WIDTH: u16 = 170;
    /// from datasheet
    pub const HEIGHT: u16 = 320;
    /// from experimentation
    pub const OFFSET_X: u16 = 35;
    /// from experimentation
    pub const OFFSET_Y: u16 = 0;
    /// User choice: landscape mode + puts the button on the right side
    pub const ORIENTATION: Orientation = Orientation::new().rotate(Rotation::Deg90);
}
pub const DISPLAY_PIXEL_COUNT: usize =
    lilygo_display_config::WIDTH as usize * lilygo_display_config::HEIGHT as usize;

pub fn setup(
    peripherals: esp_hal::peripherals::Peripherals,
) -> (Display, (Button, Button), I2c<'static, esp_hal::Blocking>) {
    // == DISPLAY SETUP ==
    let display = {
        let lcd_out_config = OutputConfig::default();
        // Pinout: Check T-DISPLAY-S3 pinout documentation
        let lcd_d0: esp_hal::gpio::Output<'_> = esp_hal::gpio::Output::new(
            peripherals.GPIO39,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_d1 = esp_hal::gpio::Output::new(
            peripherals.GPIO40,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_d2 = esp_hal::gpio::Output::new(
            peripherals.GPIO41,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_d3 = esp_hal::gpio::Output::new(
            peripherals.GPIO42,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_d4 = esp_hal::gpio::Output::new(
            peripherals.GPIO45,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_d5 = esp_hal::gpio::Output::new(
            peripherals.GPIO46,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_d6 = esp_hal::gpio::Output::new(
            peripherals.GPIO47,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_d7 = esp_hal::gpio::Output::new(
            peripherals.GPIO48,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );

        let lcd_dc = esp_hal::gpio::Output::new(
            peripherals.GPIO7,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );
        let lcd_wr = esp_hal::gpio::Output::new(
            peripherals.GPIO8,
            esp_hal::gpio::Level::High,
            lcd_out_config,
        );
        let lcd_rd = esp_hal::gpio::Output::new(
            peripherals.GPIO9,
            esp_hal::gpio::Level::High,
            lcd_out_config,
        );
        let lcd_rst = esp_hal::gpio::Output::new(
            peripherals.GPIO5,
            esp_hal::gpio::Level::High,
            lcd_out_config,
        );

        let lcd_power_on = esp_hal::gpio::Output::new(
            peripherals.GPIO15,
            esp_hal::gpio::Level::High,
            lcd_out_config,
        );
        let lcd_bl = esp_hal::gpio::Output::new(
            peripherals.GPIO38,
            esp_hal::gpio::Level::High,
            lcd_out_config,
        );
        let lcd_cs = esp_hal::gpio::Output::new(
            peripherals.GPIO6,
            esp_hal::gpio::Level::Low,
            lcd_out_config,
        );

        lcd_display_setup(
            (
                lcd_d0, lcd_d1, lcd_d2, lcd_d3, lcd_d4, lcd_d5, lcd_d6, lcd_d7,
            ),
            lcd_dc,
            lcd_wr,
            lcd_rd,
            lcd_rst,
            lcd_power_on,
            lcd_bl,
            lcd_cs,
        )
    };
    // == BUTTON SETUP ==
    let config = ButtonConfig::default();
    let (button_0, button_1) = {
        let input_config = InputConfig::default().with_pull(esp_hal::gpio::Pull::Up);
        (
            esp_hal::gpio::Input::new(peripherals.GPIO0, input_config),
            esp_hal::gpio::Input::new(peripherals.GPIO14, input_config),
        )
    };
    let button_0: button_driver::Button<_, WrappedInstant> =
        button_driver::Button::new(GPIODriver { pin: button_0 }, config);
    let button_1: button_driver::Button<_, WrappedInstant> =
        button_driver::Button::new(GPIODriver { pin: button_1 }, config);

    // == I2C SETUP ==
    let config = esp_hal::i2c::master::Config::default().with_frequency(Rate::from_khz(100));

    // You need to configure the driver during initialization:
    let i2c = I2c::new(peripherals.I2C0, config)
        .unwrap() //TODO: handle error - return Result
        .with_sda(peripherals.GPIO43)
        .with_scl(peripherals.GPIO44);

    // Return the items
    (display, (button_0, button_1), i2c)
}

/// Creates and initializes the display
/// A small wrapper around the mipidsi display builder
///
/// Can panic if the display cannot be initialized
fn lcd_display_setup(
    data: (
        PinOut,
        PinOut,
        PinOut,
        PinOut,
        PinOut,
        PinOut,
        PinOut,
        PinOut,
    ),
    dc: PinOut,
    wr: PinOut,
    rd: PinOut,
    rst: PinOut,
    mut power_on: PinOut,
    mut backlight: PinOut,
    mut child_select: PinOut,
) -> Display {
    power_on.set_high();
    backlight.set_high();
    child_select.set_low();
    let bus = Generic8BitBus::new(data);
    let di = ParallelInterface::new(bus, dc, wr);

    let mut delay = Delay::new();

    // inspired by https://github.com/almindor/mipidsi/blob/master/examples/spi-st7789-rpi-zero-w/src/main.rs
    match Builder::new(ST7789, di)
        .reset_pin(rst)
        .invert_colors(ColorInversion::Inverted)
        .display_size(lilygo_display_config::WIDTH, lilygo_display_config::HEIGHT)
        .display_offset(
            lilygo_display_config::OFFSET_X,
            lilygo_display_config::OFFSET_Y,
        )
        .orientation(lilygo_display_config::ORIENTATION)
        .init(&mut delay)
    {
        Ok(display) => {
            log::info!("Display initialized");
            display
        }
        Err(err) => {
            log::error!("Display initialization error: {:?}", err);
            panic!("Impossible to initialize display");
        }
    }
}

// == GPIO WRAPPER FOR BUTTONS ==
pub struct GPIODriver {
    pin: esp_hal::gpio::Input<'static>,
}

impl button_driver::PinWrapper for GPIODriver {
    fn is_high(&mut self) -> bool {
        self.pin.is_high()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WrappedInstant {
    instant: Instant,
}

impl InstantProvider<core::time::Duration> for WrappedInstant {
    fn now() -> Self {
        Self {
            instant: Instant::now(),
        }
    }
}

impl core::ops::Sub for WrappedInstant {
    type Output = core::time::Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        let ms = (self.instant - rhs.instant).as_micros();
        core::time::Duration::from_micros(ms)
    }
}

mod tests {
    use super::*;
    use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
    use sensirion_SLF::SensorCommunication;
    fn _test_display(display: &mut Display) {
        log::debug!("Starting test of display");
        let delay = Delay::new();
        display.clear(Rgb565::RED).unwrap();
        delay.delay_millis(300u32);
        log::info!("Starting test images");
        mipidsi::TestImage::new().draw(display).unwrap();
    }

    fn _test_i2c(i2c: esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>) {
        let mut sensor: sensirion_SLF::slf3_driver::Slf3sDriver<
            _,
            sensirion_SLF::models::SLF3S_0600F,
        > = sensirion_SLF::slf3_driver::Slf3sDriver::new(i2c);

        //sensor. ;
        const DEVICE_ADDR: u8 = 0x77;
        //let write_buffer = [0xAA];
        //let mut read_buffer = [0u8; 22];
        let delay = Delay::new();
        loop {
            log::info!("Sending I2C command");

            match sensor.read_product_id() {
                Ok((product_number, serial_number)) => {
                    log::info!("PN: {product_number:?}, SN: {serial_number:x}")
                }
                Err(e) => log::error!("I2C write error: {e:?}"),
            }
            delay.delay_millis(500);
        }
    }
}
