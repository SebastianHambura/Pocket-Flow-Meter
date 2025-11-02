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
use kolibri_embedded_gui::button::{self, Button};
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::smartstate::SmartstateProvider;
use kolibri_embedded_gui::style::medsize_rgb565_style;
use kolibri_embedded_gui::toggle_switch::ToggleSwitch;
use kolibri_embedded_gui::ui::Ui;
use log::info;

use embedded_charts::prelude::*;
use embedded_graphics::prelude::*;

use button_driver::{ButtonConfig, InstantProvider, PinWrapper};

use embedded_charts::data::{
    OverflowMode, PointRingBuffer, RingBuffer, RingBufferConfig, RingBufferEvent,
};

use micromath::F32Ext;

use crate::gui::SensorWidget;

mod gui;
mod lilygo_hal;
mod sensor;

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

fn test_i2c(mut i2c: esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>) {
    let mut sensor = sensirion_SLF::SLF3S::new(i2c);
   
    //sensor. ;
    const DEVICE_ADDR: u8 = 0x77;
    //let write_buffer = [0xAA];
    //let mut read_buffer = [0u8; 22];
    let delay = Delay::new();
    loop {
        log::info!("Sending I2C command");

        match  sensor.read_product_id() {
            Ok((product_number, serial_number)) => log::info!("PN: {product_number:x}, SN: {serial_number:x}"),
            Err(e) => log::error!("I2C write error: {e:?}"),
        }
        delay.delay_millis(500);
    }
}
pub struct GPIODriver {
    pin: esp_hal::gpio::Input<'static>
}

impl PinWrapper for GPIODriver<> {
    fn is_high(&mut self) -> bool {
        self.pin.is_high()
    }
}

#[derive(Clone)]
pub struct WrappedInstant {
    instant: Instant
}

impl InstantProvider for WrappedInstant {
    fn now() -> Self {
        Self{ instant: Instant::now() }
    }
}

impl core::ops::Sub for WrappedInstant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.instant - rhs.instant
    }
}

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(size: 32 * 1024);
    // generator version: 0.5.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let (mut display, (button_0, button_1), i2c) = lilygo_hal::setup(peripherals);
    let wrapped_button = GPIODriver{pin: button_0} ;
    let mut button_driver: button_driver::Button<_, > = button_driver::Button::new(wrapped_button, ButtonConfig::default()) ;
    let mut sensor = sensor::Sensor::new(i2c);
    //test_display(&mut display);

    display.clear(Rgb565::RED).unwrap();

    let mut toggle_0 = false;
    let mut toggle_1 = false;

    // clear the background only once
    Ui::new_fullscreen(&mut display, medsize_rgb565_style())
        .clear_background()
        .unwrap();

    let mut buffer = [Rgb565::new(0, 0, 0); lilygo_hal::DISPLAY_PIXEL_COUNT];

    let mut sensor_widget: SensorWidget<100> = gui::SensorWidget::new();

    let delay = Delay::new();
    let mut i = 0;
    loop {
        button_driver.tick() ;
        if let Some(meas) = sensor.get_measurement(i as f32) {
            sensor_widget.new_sensor_value(meas);
        }
        log::info!("{:?}", sensor.get_ID()) ;
        

        // Use chronological iterator for proper time ordering
        let mut chart_data = sensor_widget.get_static_data();

        // Calculate moving average
        // if let Some(avg) = streaming_buffer.moving_average(20) {
        //     display_average(avg);
        // }

        toggle_0 = button_0.is_low();
        toggle_1 = button_1.is_low();

        info!("Hello world!");
        let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
        ui.set_buffer(&mut buffer);
        // restart the counter at the start (or end) of the loop
        ui.add(Label::new("Flow meter").with_font(ascii::FONT_10X20));

        ui.add_horizontal(ToggleSwitch::new(&mut toggle_1).height(15));
        ui.add(Label::new("Button 1").with_font(ascii::FONT_6X10));

        let allocation = ui.allocate_space(Size::new(300, 80));

        ui.new_row();
        ui.add_horizontal(ToggleSwitch::new(&mut toggle_0).height(15));
        ui.add(Label::new("Do measurements").with_font(ascii::FONT_6X10));
        let result: Result<(), u32> = match allocation {
            Ok(res) => {
                sensor_widget.chart(res.area, &mut display);
                Ok(())
            }
            Err(err) => {
                log::error!("{err:?}");
                Ok(())
            }
        };

        i += 1;
        delay.delay_millis(100);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
