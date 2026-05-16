#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_graphics::pixelcolor::Rgb565;
use esp_hal::clock::CpuClock;
use esp_hal::time::Instant;
use esp_hal::{delay::Delay, main};

use embedded_charts::prelude::*;

use esp_println::print;
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
fn panic(msg: &core::panic::PanicInfo) -> ! {
    print!("PANIC: {msg}");
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(size: 32 * 1024);
    // generator version: 0.5.0

    // === Init hardware ===
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let (mut display, (mut button_0, mut button_1), i2c) = lilygo_hal::setup(peripherals);
    display.clear(Rgb565::RED).unwrap();

    // === Init app state ===
    let delay = Delay::new();
    let mut state = sensor::State::Water;
    let mut sensor_name: String<15> = String::new();

    // === Init sensor ===
    delay.delay_millis(50);
    let mut i2c_port = i2c;
    let mut slf_sensor = loop {
        match sensor::auto_detect_sensor(i2c_port) {
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
    sensor::set_sensor_state(&mut slf_sensor, &state);

    let _ = write!(&mut sensor_name, "{}", slf_sensor.name());
    utils::fill_string(&mut sensor_name, ' ');

    // === Init UI state ===
    let mut fbuf = embedded_graphics_framebuf::FrameBuf::new(
        [Rgb565::WHITE; lilygo_hal::DISPLAY_PIXEL_COUNT],
        lilygo_hal::lilygo_display_config::WIDTH as usize,
        lilygo_hal::lilygo_display_config::HEIGHT as usize,
    );
    
    let mut gui = gui::Ui::new(fbuf.bounding_box(), &sensor_name, style::UiStyle::default());
    gui.set_flow_unit(slf_sensor.flow_unit());
    let mut update_plot = true;

    // === Main loop ===
    let mut history = history::History::<350>::new("ms", slf_sensor.flow_unit(), None);
    let experiment_start = Instant::now();
    log::info!("Starting main loop");
    loop {
        // === Do button handling ===
        button_0.tick();
        button_1.tick();
        if button_0.is_clicked() {
            state.toggle();
            sensor::set_sensor_state(&mut slf_sensor, &state);
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

        // === Handle UI logic ===
        gui.tick_update(update_plot, state.as_str());
        if update_plot {
            let newest = history.get_newest().map(|point| point.y);
            // update the chart and the value widget
            gui.chart_update(&history); //TODO: Maybe don't copy the whole ringbuffer every tick?
            gui.sensor_value_update(newest);
        };
        if let Err(err) = gui.draw(&mut fbuf) {
            log::error!("Failed to draw UI: {:?}", err);
        }

        // === House-keeping
        // Draw the framebuffer to the display
        if let Err(err) = display.fill_contiguous(&fbuf.bounding_box(), fbuf.data.iter().cloned()) {
            log::error!("Failed to fill display: {:?}", err);
        }

        // We need to reset the buttons
        button_0.reset();
        button_1.reset();
        delay.delay_millis(100);
    }
}
