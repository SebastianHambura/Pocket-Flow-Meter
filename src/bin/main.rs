#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_graphics::mono_font::ascii;
use embedded_graphics::pixelcolor::Rgb565;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{InputConfig, OutputConfig, OutputPin, Pin};
use esp_hal::time::{Duration, Instant};
use esp_hal::{delay::Delay, main};
use kolibri_embedded_gui::button::Button;
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::smartstate::SmartstateProvider;
use kolibri_embedded_gui::style::medsize_rgb565_style;
use kolibri_embedded_gui::toggle_switch::ToggleSwitch;
use kolibri_embedded_gui::ui::Ui;
use log::info;

use embedded_graphics::{prelude::*};

mod lilygo_hal;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn test_display(display: &mut lilygo_hal::Display) {
    log::debug!("Starting test of display");
    let delay = Delay::new();
    display.clear(Rgb565::RED).unwrap();
    delay.delay_millis(300u32);
    log::info!("Starting test images");
    mipidsi::TestImage::new().draw(display).unwrap();
}

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    // generator version: 0.5.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let (mut display, (button_0, button_1)) = lilygo_hal::setup(peripherals);
    //test_display(&mut display);

    display.clear(Rgb565::RED).unwrap();

    let mut toggle_0 = false;
    let mut toggle_1 = false;

    // clear the background only once
    Ui::new_fullscreen(&mut display, medsize_rgb565_style())
        .clear_background()
        .unwrap();

    let mut buffer = [Rgb565::new(0, 0, 0);
        lilygo_hal::DISPLAY_PIXEL_COUNT];
    loop {
        toggle_0 = button_0.is_low();
        toggle_1 = button_1.is_low();

        info!("Hello world!");
        let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
        ui.set_buffer(&mut buffer);
        // restart the counter at the start (or end) of the loop
        ui.add(Label::new("Basic Example").with_font(ascii::FONT_10X20));

        ui.add_horizontal(ToggleSwitch::new(&mut toggle_1));
        ui.add(Label::new("Button 1").with_font(ascii::FONT_10X20));

        ui.new_row();

        ui.add_horizontal(ToggleSwitch::new(&mut toggle_0));
        ui.add(Label::new("Button 0").with_font(ascii::FONT_10X20));
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
