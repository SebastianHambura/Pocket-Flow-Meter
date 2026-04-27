//! A widget to display a value with a label, and an optional unit.
//!
//! Custom widget, uses the embedded_bitmap_fonts crate to display some big numbers
use core::fmt::Write;
use embedded_bitmap_fonts::terminus::FONT_8x14_BOLD;
use embedded_charts::prelude::*;
use embedded_graphics::text::{Baseline, Text};

use crate::utils::color_converter::BinaryToRgb565;

pub struct ValueWithLabelWidget<const VARIABLE_CHARS: usize, const CONSTANT_CHARS: usize> {
    pub value_str: String<VARIABLE_CHARS>,
    label_str: String<CONSTANT_CHARS>,
    const_already_drawn: bool,
}

impl<const VARIABLE_CHARS: usize, const CONSTANT_CHARS: usize>
    ValueWithLabelWidget<VARIABLE_CHARS, CONSTANT_CHARS>
{
    pub fn new(label: &str) -> Self {
        let value_str = String::<VARIABLE_CHARS>::new();
        let mut label_str = String::<CONSTANT_CHARS>::new();
        if label_str.push_str(label).is_err() {
            log::warn!(
                "Error - Trying to push {} into String with {} chars )",
                label,
                CONSTANT_CHARS
            );
        }

        Self {
            value_str,
            label_str,
            const_already_drawn: false,
        }
    }

    pub fn update_value(&mut self, value: f32) {
        //TODO: make this smarter
        self.value_str.clear();
        match write!(&mut self.value_str, "{:05.1}", value) {
            Ok(_) => (),
            Err(err) => log::warn!("{} (value: {})", err, value),
        };
    }

    pub fn draw<T>(&mut self, start_point: Point, display: &mut T) -> RenderResult<()>
    where
        T: DrawTarget<Color = Rgb565> + embedded_graphics::geometry::OriginDimensions,
    {
        use embedded_bitmap_fonts::{terminus::FONT_14x28_BOLD, TextStyle};
        let larger_font = FONT_14x28_BOLD.pixel_double();

        let mut style = TextStyle::new(&larger_font, BinaryColor::On);

        let mut point = start_point;
        point.y -= 7; // Same as line 63

        //self.flow_text.draw(point, &style, display)?;
        let mut value_text = Text::with_baseline(&self.value_str, point, style, Baseline::Top);

        // Paint the background
        let mut background = value_text.bounding_box();
        // These values are probably FONT and FONT size dependent
        background = background.resized_height(
            background.size.height - 7,
            embedded_graphics::geometry::AnchorY::Bottom,
        );
        background = background.resized_height(
            background.size.height - 11,
            embedded_graphics::geometry::AnchorY::Top,
        );

        display.fill_solid(&background, Rgb565::BLACK);

        // Convert from Binary to some colours
        let mut adapter = BinaryToRgb565::new(
            display,
            Rgb565::RED, // ON color
            None,        // OFF = transparent (recommended)
        );
        //log::info!("Drawing value '{}' at {:?}", self.value_str, point);

        value_text.draw(&mut adapter);

        let const_style = TextStyle::new(&FONT_8x14_BOLD, BinaryColor::On);
        // label goes to the right of the value, and a bit below
        let label_point = Point {
            x: background.top_left.x + background.size.width as i32 + 5, // 5 pixels to the right of the value
            y: background.top_left.y + background.size.height as i32 / 2
                - const_style.font.height() as i32 / 2, // 10 pixels below the top of the value
        };
        let const_text = Text::new(&self.label_str, label_point, const_style);
        //log::info!("Drawing label '{}' at {:?}", self.label_str, label_point);
        const_text.draw(&mut adapter);

        Ok(())
    }
}

