#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_graphics::pixelcolor::Rgb565;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{OutputConfig, OutputPin, Pin};
use esp_hal::time::{Duration, Instant};
use esp_hal::{delay::Delay, main};
use log::info;

use embedded_graphics::{pixelcolor::Rgb666, prelude::*};
use mipidsi::interface::{Generic8BitBus, OutputBus, ParallelInterface, SpiInterface};
use mipidsi::models::ST7789;
use mipidsi::options::ColorInversion;
use mipidsi::{Builder, Display}; // Provides the required color type

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn lcd_display_setup(
    peripherals: esp_hal::peripherals::Peripherals,
) -> mipidsi::Display<
    mipidsi::interface::ParallelInterface<
        mipidsi::interface::Generic8BitBus<
            esp_hal::gpio::Output<'static>,
            esp_hal::gpio::Output<'static>,
            esp_hal::gpio::Output<'static>,
            esp_hal::gpio::Output<'static>,
            esp_hal::gpio::Output<'static>,
            esp_hal::gpio::Output<'static>,
            esp_hal::gpio::Output<'static>,
            esp_hal::gpio::Output<'static>,
        >,
        esp_hal::gpio::Output<'static>,
        esp_hal::gpio::Output<'static>,
    >,
    mipidsi::models::ST7789,
    esp_hal::gpio::Output<'static>,
> {
    let config = OutputConfig::default();

    // Pinout: Check T-DISPLAY-S3 pinout documentation
    let lcd_d0 = esp_hal::gpio::Output::new(peripherals.GPIO39, esp_hal::gpio::Level::Low, config);
    let lcd_d1 = esp_hal::gpio::Output::new(peripherals.GPIO40, esp_hal::gpio::Level::Low, config);
    let lcd_d2 = esp_hal::gpio::Output::new(peripherals.GPIO41, esp_hal::gpio::Level::Low, config);
    let lcd_d3 = esp_hal::gpio::Output::new(peripherals.GPIO42, esp_hal::gpio::Level::Low, config);
    let lcd_d4 = esp_hal::gpio::Output::new(peripherals.GPIO45, esp_hal::gpio::Level::Low, config);
    let lcd_d5 = esp_hal::gpio::Output::new(peripherals.GPIO46, esp_hal::gpio::Level::Low, config);
    let lcd_d6 = esp_hal::gpio::Output::new(peripherals.GPIO47, esp_hal::gpio::Level::Low, config);
    let lcd_d7 = esp_hal::gpio::Output::new(peripherals.GPIO48, esp_hal::gpio::Level::Low, config);

    let dc = esp_hal::gpio::Output::new(peripherals.GPIO7, esp_hal::gpio::Level::Low, config);
    let wr = esp_hal::gpio::Output::new(peripherals.GPIO8, esp_hal::gpio::Level::High, config);
    let rd = esp_hal::gpio::Output::new(peripherals.GPIO9, esp_hal::gpio::Level::High, config);
    let rst = esp_hal::gpio::Output::new(peripherals.GPIO5, esp_hal::gpio::Level::High, config);

    // other pins
    // power on: on
    // bl = backlight (?): on
    // cs = child select: low (active low)
    let mut lcd_power_on =
        esp_hal::gpio::Output::new(peripherals.GPIO15, esp_hal::gpio::Level::High, config);
    let mut lcd_bl =
        esp_hal::gpio::Output::new(peripherals.GPIO38, esp_hal::gpio::Level::High, config);
    let mut lcd_cs =
        esp_hal::gpio::Output::new(peripherals.GPIO6, esp_hal::gpio::Level::Low, config);
    //lcd_cs.set_low(); ;
    //lcd_power_on.set_high();
    //lcd_bl.set_high();

    let bus = Generic8BitBus::new((
        lcd_d0, lcd_d1, lcd_d2, lcd_d3, lcd_d4, lcd_d5, lcd_d6, lcd_d7,
    ));
    let di = ParallelInterface::new(bus, dc, wr);

    let mut delay = Delay::new();

    // inspired by https://github.com/almindor/mipidsi/blob/master/examples/spi-st7789-rpi-zero-w/src/main.rs
    Builder::new(ST7789, di)
        .reset_pin(rst)
        .invert_colors(ColorInversion::Inverted)
        .init(&mut delay)
        .unwrap()
}

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    // generator version: 0.5.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut delay = Delay::new();

    let mut buffer = [0_u8; 512];
    let mut display = lcd_display_setup(peripherals);

    display.clear(Rgb565::RED).unwrap();
    let colors = [
        Rgb565::RED,
        Rgb565::GREEN,
        Rgb565::BLUE,
        Rgb565::YELLOW,
        Rgb565::CYAN,
        Rgb565::MAGENTA,
        Rgb565::WHITE,
        Rgb565::BLACK,
    ];
    let mut colors = colors.iter().cycle();
    loop {
        info!("Hello world!");
        let new_color = colors.next().unwrap();
        display.clear(*new_color).unwrap();
        delay.delay_millis(1000);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
