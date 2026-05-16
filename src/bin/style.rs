//! This module contains everything related to the styling of the UI, such as colors and fonts.
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

pub struct UiStyle {
    pub background_color: Rgb565,
    pub foreground_color: Rgb565,
    pub plot_color: Rgb565,
}

impl UiStyle {
    pub fn default() -> Self {
        Self {
            background_color: Rgb565::WHITE,
            foreground_color: Rgb565::BLACK,
            plot_color: Rgb565::BLUE,
        }
    }
}
