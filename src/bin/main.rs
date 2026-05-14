#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_graphics::pixelcolor::Rgb565;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::I2c;
use esp_hal::time::Instant;
use esp_hal::{delay::Delay, main};

use embedded_charts::prelude::*;

use button_driver::{ButtonConfig, InstantProvider, PinWrapper};

use sensirion_SLF::models::SLF3S_0600F;
use sensirion_SLF::slf3_driver::Slf3sDriver;
use sensirion_SLF::{SensorCommunication, SensorInformation};

use core::fmt::Write;

mod gui;
mod history;
mod lilygo_hal;
mod sensor;
mod style;
mod widgets;

mod utils;

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

fn set_sensor_state<Sensor>(sensor: &mut Sensor, state: &State)
where
    Sensor: sensirion_SLF::SensorCommunication,
{
    if let Err(err) = sensor.stop_measurement() {
        log::warn!("Problem stopping measurmenent: {err:?}")
    };
    Delay::new().delay_micros(600);
    //delay.delay_micros(600);
    if let Err(err) = match state {
        State::Water => sensor.start_continuous_measurement_water(),
        State::Ethanol => sensor.start_continuous_measurement_alcohol(),
    } {
        log::warn!("Problem stopping measurmenent: {err:?}")
    };
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

    let delay = Delay::new();
    let mut sensor_name: String<15> = String::new();

    delay.delay_millis(50);

    let mut i2c_port = i2c;
    let mut slf_sensor = loop {
        match sensor::auto_dectect_sensor(i2c_port) {
            Ok(sensor) => {
                break sensor;
            }
            Err((i2c, err)) => {
                i2c_port = i2c;
                log::error!("{err}");
                delay.delay_millis(250);
            }
        }
    };

    set_sensor_state(&mut slf_sensor, &state);

    let _ = write!(&mut sensor_name, "{}", slf_sensor.name());
    fill_string(&mut sensor_name, ' ');

    //let mut sensor = sensor::Sensor::new(i2c);

    //test_display(&mut display);

    display.clear(Rgb565::RED).unwrap();

    let mut update_plot = true;

    let mut fbuf = embedded_graphics_framebuf::FrameBuf::new([Rgb565::WHITE; 320 * 170], 320, 170);

    // UI components stuff
    // let mut value_widget =
    //     widgets::value::ValueWithLabelWidget::<5, 16>::new(slf_sensor.flow_unit());
    // let mut streaming_chart = widgets::chart::StreamedDataPlot::<350>::new(
    //     Rgb565::CSS_STEEL_BLUE,
    //     Some(Rgb565::WHITE),
    //     AxisPosition::Left,
    // );

    let mut history = history::History::<350>::new("ms", slf_sensor.flow_unit(), None);

    let mut gui = gui::Ui::new(fbuf.bounding_box(), &sensor_name, style::UiStyle::default());
    gui.set_flow_unit(slf_sensor.flow_unit());

    let experiment_start = Instant::now();
    log::info!("Starting main loop");
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
        let measure = slf_sensor.read_measurement();
        match measure {
            Ok((flow, temp, signal)) => {
                // Convert raw value into the proper physical unit
                let flow: f32 = flow.into();
                let real_flow = flow / slf_sensor.flow_factor();

                history.push(Point2D {
                    x: (experiment_start.elapsed().as_millis() as f32) / 1000.0,
                    y: real_flow,
                });
            }
            Err(err) => log::error!("[mes] {:?}", err),
        }

        {
            let text_str = match state {
                State::Water => "H2O ",
                State::Ethanol => "EtOH",
            };
            // === Create and update the screen ===
            fbuf.clear(Rgb565::WHITE);
            gui.tick_update(update_plot, text_str);

            if update_plot {
                gui.chart_update(&history); // Maybe don't copy the whole ringbuffer every tick?
                let newest = history.get_newest().map(|point| point.y);
                gui.sensor_value_update(newest);
            };
            gui.draw(&mut fbuf);

            // === Chart row ===
            let chart_allocation = Rectangle::new(
                Point { x: 0, y: 20 + 10 },
                Size {
                    width: fbuf.width() as u32,
                    height: 95,
                },
            );

            // let _ = streaming_chart.draw_chart(&mut fbuf);

            // === Bottom row ===
            let margin = 5;
            let bottom_row = Point::new(0, chart_allocation.bottom_right().unwrap().y + margin);

            let mut point = bottom_row;
            // let icon = size18px::navigation::NavArrowLeft::new(Rgb565::BLACK);
            // let img = Image::new(&icon, point);
            // img.draw(&mut fbuf);
            // point.x += 20; //18px + margin

            // let text_str = match state {
            //     State::Water => {
            //         "H2O "
            //         //ui.add_horizontal(spacer::Spacer::new(Size::new(2*20, 0))); // Creating horizontal space
            //     }
            //     State::Ethanol => "EtOH",
            // };
            // point.y += 20;
            // let name_style = MonoTextStyle::new(&ascii::FONT_10X20, Rgb565::BLACK);
            // let text = Text::new(&text_str, point, name_style);
            // text.draw(&mut fbuf);
            // point.y -= 20;
            // point.x += text.bounding_box().size.width as i32;

            // let icon = size18px::navigation::NavArrowRight::new(Rgb565::BLACK);
            // let img = Image::new(&icon, point);
            // img.draw(&mut fbuf);
            // point.x += 20; //18px + margin

            //value_widget.draw(point, &mut fbuf).unwrap();

            display.fill_contiguous(&fbuf.bounding_box(), fbuf.data.iter().cloned());
        }

        // === House-keeping

        // We need to reset the buttons
        button_0.reset();
        button_1.reset();
        delay.delay_millis(100);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
