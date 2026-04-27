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
use esp_hal::i2c::master::I2c;
use esp_hal::time::Instant;
use esp_hal::{delay::Delay, main};
use kolibri_embedded_gui::label::Label;
use kolibri_embedded_gui::ui::Ui;

use embedded_charts::prelude::*;

use button_driver::{ButtonConfig, InstantProvider, PinWrapper};

use sensirion_SLF::models::SLF3S_0600F;
use sensirion_SLF::slf3_driver::Slf3sDriver;
use sensirion_SLF::SensorCommunication;

use core::fmt::Write;

use crate::sensor::Measurement;

mod gui;
mod lilygo_hal;
mod sensor;
mod utils;
mod widgets;

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
    let mut sensor: Slf3sDriver<_, SLF3S_0600F> = sensirion_SLF::slf3_driver::Slf3sDriver::new(i2c);

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

#[derive(Debug)]
enum State {
    Water,
    Ethanol,
}

type BlockingI2C = I2c<'static, esp_hal::Blocking>;
enum SensorType {
    Real(sensirion_SLF::slf3_driver::Slf3sDriver<BlockingI2C, sensirion_SLF::models::SLF3S_0600F>),
    Fake(sensirion_SLF::fake_sensor::FakeSLF3),
}

fn set_sensor_state(sensor: &mut SensorType, state: &State) {
    match sensor {
        SensorType::Real(slf3_s) => {
            if let Err(err) = slf3_s.stop_measurement() {
                log::warn!("Problem stopping measurmenent: {err:?}")
            };
            Delay::new().delay_micros(600);
            //delay.delay_micros(600);
            if let Err(err) = match state {
                State::Water => slf3_s.start_continuous_measurement_water(),
                State::Ethanol => slf3_s.start_continuous_measurement_alcohol(),
            } {
                log::warn!("Problem stopping measurmenent: {err:?}")
            };
        }
        SensorType::Fake(fake_slf3) => {
            log::trace!("Nothing to do for the dake sensor")
        }
    }
    log::info!("Switched to {state:?} mode");
}

fn fill_string<const N: usize>(str: &mut String<N>, c: char) {
    let n = str.len();
    let missing_char = N - n;
    for _ in 0..missing_char {
        let _ = str.push(c);
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
    let mut config = ButtonConfig::default();
    // config.mode = button_driver::Mode::PullUp; PullUp is default

    let mut button_0: button_driver::Button<_, WrappedInstant> =
        button_driver::Button::new(GPIODriver { pin: button_0 }, config);
    let mut button_1: button_driver::Button<_, WrappedInstant> =
        button_driver::Button::new(GPIODriver { pin: button_1 }, config);

    let mut state = State::Water;
    let mut slf_sensor: Slf3sDriver<_, SLF3S_0600F> =
        sensirion_SLF::slf3_driver::Slf3sDriver::new(i2c);

    if let Err(err) = slf_sensor.soft_reset() {
        log::warn!("Error while doing a soft reset: {err:?}")
    };

    let delay = Delay::new();
    let mut sensor_name: String<15> = String::new();

    delay.delay_millis(50);

    let mut slf_sensor = match slf_sensor.read_product_id() {
        Ok(val) => {
            log::info!("Sensor detected !");
            log::info!("Product ID: {:?}", val);
            log::info!("Sensor ID: {}", val.0.raw_value());
            match write!(
                &mut sensor_name,
                "SLF{}-{}-{}-{}",
                val.0.liquid_flow_sensor(),
                val.0.product_family(),
                val.0.subtype(),
                val.0.revision_number()
            ) {
                Ok(_) => (),
                Err(err) => log::warn!("{} (value: {:?})", err, val),
            };
            SensorType::Real(slf_sensor)
        }
        Err(err) => {
            log::error!("No sensor detected: {:?}", err);
            log::info!("Continuing with fake sensor");
            write!(&mut sensor_name, "SLF-[DUMMY]",)
                .expect("it's safe to write this &str into the String");
            SensorType::Fake(sensirion_SLF::fake_sensor::FakeSLF3::new(0, 100, 20))
        }
    };
    set_sensor_state(&mut slf_sensor, &state);
    fill_string(&mut sensor_name, ' ');

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

    let mut buffer = [Rgb565::RED; lilygo_hal::DISPLAY_PIXEL_COUNT];
    // let mut buffer = Vec::new() ;
    // buffer.push(Rgb565::RED);
    // buffer.repeat(lilygo_hal::DISPLAY_PIXEL_COUNT);

    //let mut plot_buffer = [Rgb565::new(0, 0, 0); 320*100];

    let mut update_plot = true;

    let mut i = 0;
    let mut fbuf = embedded_graphics_framebuf::FrameBuf::new([Rgb565::WHITE; 320 * 100], 320, 100);

    // UI components stuff
    let mut value_widget = widgets::value::ValueWithLabelWidget::<5, 16>::new("uL/min");
    let mut streaming_chart = widgets::chart::StreamedDataPlot::<256>::new(
        Rgb565::CSS_STEEL_BLUE,
        Some(Rgb565::WHITE),
        AxisPosition::Left,
    );
    loop {
        // === Do button handling ===
        button_0.tick();
        button_1.tick();
        if button_0.is_clicked() {
            match state {
                State::Water => {
                    state = State::Ethanol;
                }
                State::Ethanol => {
                    state = State::Water;
                }
            };
            set_sensor_state(&mut slf_sensor, &state);
        };

        if button_1.is_clicked() {
            update_plot = !update_plot;
            log::info!("Freeze button clicked (now update_plot = {})", update_plot);
        }

        // === Handle sensor logic ===
        let mes = match slf_sensor {
            SensorType::Real(ref mut slf3_s) => slf3_s.read_measurement(),
            SensorType::Fake(ref mut fake_slf3) => fake_slf3.read_measurement(),
        };
        match mes {
            Ok((flow, temp, signal)) => {
                // sensor_widget.new_sensor_value(Measurement::new(
                //     i as f32,
                //     values.0 as f32 / 10.0, //slf3::SLF3S::<_>::LIQUID_FLOW_RATE_SCALE_FACTOR,
                //     values.1 as f32 / 200.0, //slf3::SLF3S::<_>::TEMPERATURE_SCALE_FACTOR),
                // ));
                streaming_chart.push_point(Point2D { x: i as f32, y: flow as f32 });
                value_widget.update_value(flow as f32 / 10.0);
            }
            Err(err) => log::error!("[mes] {:?}", err),
        }

        {
            // === Create and update the screen ===
            use kolibri_embedded_gui::*;
            use kolibri_embedded_gui::{icon::*, icons::*};

            let mut ui = Ui::new_fullscreen(&mut display, style::medsize_light_rgb565_style());
            ui.draw_widget_bounds_debug(Rgb565::GREEN);
            //ui.clear_background().unwrap();
            ui.set_buffer(&mut buffer);

            // == Header row ===
            match slf_sensor {
                SensorType::Real(_) => {
                    ui.add_horizontal(IconWidget::new(size18px::actions::DoubleCheck))
                }
                SensorType::Fake(_) => ui.add_horizontal(IconWidget::new(size18px::other::NoLink)),
            };

            ui.add_horizontal(Label::new(&sensor_name).with_font(ascii::FONT_10X20));
            // Some manual fiddling to push the button to the edge
            ui.add_horizontal(spacer::Spacer::new(Size::new(5 * 20, 0))); // Creating horizontal space
                                                                          // FREEZE : 6
                                                                          // LIVE-UPDATE : 11
            if update_plot {
                ui.add(IconWidget::new(size18px::music::Play))
            } else {
                ui.add(IconWidget::new(size18px::music::Pause))
            };

            // === Chart row ===
            let chart_allocation = match ui.allocate_space(Size::new(310, 95)) {
                Ok(res) => Some(res.area),
                Err(err) => {
                    log::error!("[chart_allocation] {:?}", err);
                    None
                }
            };
            

            //let total_area = chart_allocation ;
            let total_area = chart_allocation
                .map(|rect| rect.resized_width(320, embedded_graphics::geometry::AnchorX::Left));
            ui.new_row();

            // === Bottom row ===
            ui.add_horizontal(IconWidget::new(size18px::navigation::NavArrowLeft));
            match state {
                State::Water => {
                    ui.add_horizontal(Label::new("H2O ").with_font(ascii::FONT_10X20));
                    //ui.add_horizontal(spacer::Spacer::new(Size::new(2*20, 0))); // Creating horizontal space
                }
                State::Ethanol => {
                    ui.add_horizontal(Label::new("EtOH").with_font(ascii::FONT_10X20));
                }
            }
            ui.add_horizontal(IconWidget::new(size18px::navigation::NavArrowRight));
            //ui.add_horizontal(spacer::Spacer::new(Size::new(20, 0))); // Creating horizontal space

            let legend_allocation = match ui.allocate_space(Size::new(20, 20)) {
                Ok(res) => Some(res.area),
                Err(err) => {
                    log::error!("[legend_allocation] {:?}", err);
                    None
                }
            };
            match ui.finalize() {
                Ok(_) => (),
                Err(err) => log::warn!("[finalize] {:?}", err),
            };
            // === Plotting the graph | Non-kolibri stuff ===
            // Doing all the non-kolibri drawing separately to avoid borrow issues

            if update_plot {
                if let Some(mut rect) = chart_allocation {
                    rect.top_left.y = 0;
                    let _ = streaming_chart.draw_chart(rect, &mut fbuf);
                    // match display.fill_contiguous(&rect, fbuf.data.iter().cloned()) { // Don't ask why it's .iter.cloned, but it has to be this
                    //     Ok(_) => (),
                    //     Err(err) => log::error!("{:?}", err),
                    // };
                };
                if let Some(rect) = legend_allocation {
                    //rect.top_left.y = 0;
                    //sensor_widget.legend_widget(rect, &mut fbuf);
                    value_widget.draw(rect.top_left, &mut display).unwrap();
                    // sensor_widget
                    //     .current_values_widget(rect, &mut display)
                    //     .unwrap();
                }
                //let area = Rectangle::new(Point::new(0, 0), fbuf.size());

                if let Some(rect) = total_area {
                    match display.fill_contiguous(&rect, fbuf.data.iter().cloned()) {
                        Ok(a) => (),
                        Err(err) => log::error!("{:?}", err),
                    };
                }
            }
        }

        // === House-keeping
        i += 1;
        // We need to reset the buttons
        button_0.reset();
        button_1.reset();
        delay.delay_millis(100);

        // let area = Rectangle::new(Point::new(0, 0), fbuf.size());
        // log::info!("{:?}", area) ;
        // match display.fill_contiguous(&area, fbuf.data.iter().cloned()) {
        //     Ok(a) => (),
        //     Err(err) => log::error!("{:?}", err),
        // };
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
