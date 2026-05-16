//! A widget to display a value with a label, and an optional unit.
//!
//! Custom widget, uses the embedded_bitmap_fonts crate to display some big numbers
use anyhow::Context;
use core::fmt::Write;
use embedded_bitmap_fonts::terminus::FONT_8x14_BOLD;
use embedded_charts::prelude::*;
use embedded_graphics::text::{Baseline, Text};

use crate::utils::color_converter::BinaryToRgb565;

pub struct ValueIndicatorWidget<const VALUE_STR_SIZE: usize, const LABEL_STR_SIZE: usize> {
    position: Point,
    value_str: String<VALUE_STR_SIZE>,
    label_str: String<LABEL_STR_SIZE>,

    background_color: Rgb565,
    text_color: Rgb565,
}

impl<const VALUE_STR_SIZE: usize, const LABEL_STR_SIZE: usize>
    ValueIndicatorWidget<VALUE_STR_SIZE, LABEL_STR_SIZE>
{
    pub fn new(position: Point, label: &str, background_color: Rgb565, text_color: Rgb565) -> Self {
        let mut label_str = String::new();
        if label_str.push_str(label).is_err() {
            log::warn!(
                "Error - Trying to push {} into String with {} chars",
                label,
                LABEL_STR_SIZE
            );
        }
        Self {
            position,
            value_str: String::new(),
            label_str,
            background_color,
            text_color
        }
    }

    pub fn set_label(&mut self, label: &str) {
        self.label_str.clear();
        if self.label_str.push_str(label).is_err() {
            log::warn!(
                "Error - Trying to push {} into String with {} chars )",
                label,
                LABEL_STR_SIZE
            );
        }
    }

    pub fn update_value(&mut self, value: Option<f32>) -> anyhow::Result<()> {
        //TODO: make this smarter
        self.value_str.clear();
        match value {
            Some(value) => write!(&mut self.value_str, "{:05.1}", value),
            None => write!(&mut self.value_str, "---.-"),
        }
        .context("Trying to write a new value into the widget's string")
    }

    pub fn draw<T>(&self, target: &mut T) -> Result<(), <T>::Error>
    where
        T: DrawTarget<Color = Rgb565> + embedded_graphics::geometry::OriginDimensions,
    {
        use embedded_bitmap_fonts::{terminus::FONT_14x28_BOLD, TextStyle};
        let larger_font = FONT_14x28_BOLD.pixel_double();

        let style = TextStyle::new(&larger_font, BinaryColor::On);

        let mut point = self.position;
        let top_y_offset = 7 ;
        let bottom_y_offset = 11; 
        point.y -= top_y_offset; // Same as line 63

        //self.flow_text.draw(point, &style, display)?;
        let value_text = Text::with_baseline(&self.value_str, point, style, Baseline::Top);

        // == Paint the background ==
        let mut background = value_text.bounding_box();
        // These values are probably FONT and FONT size dependent
        background = background.resized_height(
            background.size.height.saturating_sub(top_y_offset as u32),
            embedded_graphics::geometry::AnchorY::Bottom,
        );
        background = background.resized_height(
            background.size.height.saturating_sub(bottom_y_offset as u32),
            embedded_graphics::geometry::AnchorY::Top,
        );

        target.fill_solid(&background, self.background_color);

        // === Convert from Binary to some colours === 
        let mut adapter = BinaryToRgb565::new(
            target,
            self.text_color, // ON color
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
