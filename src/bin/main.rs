#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_graphics::mono_font::{ascii, MonoFont};
use embedded_graphics::pixelcolor::Rgb565;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{InputConfig, OutputConfig, OutputPin, Pin};
use esp_hal::i2c::master::I2c;
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
use sensirion_SLF::Sensor;

use crate::gui::SensorWidget;
use crate::sensor::Measurement;

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
    let mut sensor = sensirion_SLF::slf3::SLF3S::new(i2c);

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
pub struct GPIODriver {
    pin: esp_hal::gpio::Input<'static>,
}

impl PinWrapper for GPIODriver {
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

enum State {
    Water,
    Ethanol,
}

type BlockingI2C = I2c<'static, esp_hal::Blocking>;
enum SensorType {
    Real(sensirion_SLF::slf3::SLF3S<BlockingI2C>),
    Fake(sensirion_SLF::fake_sensor::FakeSLF3),
}

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(size: 32 * 1024);
    // generator version: 0.5.0
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let (mut display, (button_0, button_1), i2c) = lilygo_hal::setup(peripherals);
    let wrapped_button = GPIODriver { pin: button_0 };
    let mut config = ButtonConfig::default();
    config.mode = button_driver::Mode::PullUp;
    let mut button_driver: button_driver::Button<_, WrappedInstant> =
        button_driver::Button::new(wrapped_button, config);

    let mut slf_sensor = sensirion_SLF::slf3::SLF3S::new(i2c);
    let mut slf_sensor = match slf_sensor.read_product_id() {
        Ok(val) => {
            log::info!("Sensor detected !");
            log::debug!("Product ID: {:?}", val);
            log::info!("Sensor ID: {}", val.0.raw_value());
            SensorType::Real(slf_sensor)
        }
        Err(err) => {
            log::error!("No sensor detected: {:?}", err);
            log::info!("Continuing with fake sensor");
            SensorType::Fake(sensirion_SLF::fake_sensor::FakeSLF3::new(0, 100, 20))
        }
    };
    //let mut sensor = sensor::Sensor::new(i2c);

    //test_display(&mut display);

    display.clear(Rgb565::RED).unwrap();

    let mut toggle_0 = false;
    let mut toggle_1 = false;

    // clear the background only once
    Ui::new_fullscreen(
        &mut display,
        kolibri_embedded_gui::style::medsize_light_rgb565_style(),
    )
    .clear_background()
    .unwrap();

    let mut buffer = [Rgb565::new(0, 0, 0); lilygo_hal::DISPLAY_PIXEL_COUNT];

    let mut sensor_widget: SensorWidget<100> = gui::SensorWidget::new();

    let mut state = State::Water;
    let delay = Delay::new();
    let mut i = 0;
    loop {
        button_driver.tick();
        let mes = match slf_sensor {
            SensorType::Real(ref mut slf3_s) => slf3_s.read_measurement(),
            SensorType::Fake(ref mut fake_slf3) => fake_slf3.read_measurement(),
        };
        match mes {
            Ok(values) => {
                sensor_widget.new_sensor_value(Measurement::new(
                    i as f32,
                    values.0 as f32,
                    values.1 as f32,
                ));
            }
            Err(err) => log::error!("{:?}", err),
        }

        // Use chronological iterator for proper time ordering
        let mut chart_data = sensor_widget.get_static_data();

        // Calculate moving average
        //if let Some(avg) = streaming_buffer.moving_average(20) {
        //    display_average(avg);
        //}
        //log::info!("{:?}", button_driver.raw_state());
        if button_driver.is_clicked() {
            match state {
                State::Water => {
                    state = State::Ethanol;
                    log::info!("Switched to Ethanol mode");
                }
                State::Ethanol => {
                    state = State::Water;
                    log::info!("Switched to Water mode");
                }
            };
        };

        //info!("Hello world!");
        {
            use kolibri_embedded_gui::*;
            use kolibri_embedded_gui::{icon::*, icons::*};
            let mut ui = Ui::new_fullscreen(&mut display, style::medsize_light_rgb565_style());
            //ui.clear_background().unwrap();
            ui.set_buffer(&mut buffer);

            // == Header row ===
            ui.add_horizontal(IconWidget::new(size18px::actions::AddCircle));
            ui.add_horizontal(Label::new("Sensirion model").with_font(ascii::FONT_10X20));
            // Some manual fiddling to push the button to the edge
            ui.add_horizontal(spacer::Spacer::new(Size::new(55, 0))); // Creating horizontal space
            ui.add(button::Button::new("Freeze"));

            let chart_allocation = ui.allocate_space(Size::new(200, 100));
            let mut legend_allocation = ui.allocate_space(Size::new(100, 50));
            // ui.sub_ui(|sub_ui| {
            //     legend_allocation = sub_ui.allocate_space(Size::new(100, 50));
            //     sub_ui.add(Label::new("99 uL/min").with_font(ascii::FONT_10X20));
            //     Ok(())
            // }).unwrap();

            //let legend_allocation = ui.allocate_space(Size::new(100, 100));
            //let mut font = ascii::FONT_10X20.clone();
            //font.character_size = Size::new(20,40) ;
            //ui.add(Label::new("99 uL/min").with_font(font));
            ui.new_row();
            ui.add_horizontal(IconWidget::new(size18px::navigation::NavArrowLeft));
            match state {
                State::Water => {
                    ui.add_horizontal(Label::new(" Water ").with_font(ascii::FONT_10X20));
                    //ui.add_horizontal(spacer::Spacer::new(Size::new(2*20, 0))); // Creating horizontal space
                }
                State::Ethanol => {
                    ui.add_horizontal(Label::new("Ethanol").with_font(ascii::FONT_10X20));
                }
            }
            ui.add_horizontal(IconWidget::new(size18px::navigation::NavArrowRight));
            ui.add_horizontal(spacer::Spacer::new(Size::new(100, 0))); // Creating horizontal space
            ui.add(button::Button::new("Switch"));

            // Doing all the non-kolibri drawing separately to avoid borrow issues
            let result: Result<(), u32> = match chart_allocation {
                Ok(res) => {
                    sensor_widget.chart(res.area, &mut display);
                    Ok(())
                }
                Err(err) => {
                    log::error!("{err:?}");
                    Ok(())
                }
            };
            let result: Result<(), u32> = match legend_allocation {
                Ok(res) => {
                    sensor_widget.legend_widget(res.area, &mut display);
                    Ok(())
                }
                Err(err) => {
                    log::error!("{err:?}");
                    Ok(())
                }
            };
        }

        i += 1;
        button_driver.reset();
        delay.delay_millis(100);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
