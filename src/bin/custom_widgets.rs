use core::fmt::Write;

use embedded_charts::prelude::*;
use embedded_graphics::mono_font::MonoTextStyle;

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

    pub fn draw<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        start_point: Point,
        style: &MonoTextStyle<Rgb565>,
        display: &mut T,
    ) -> RenderResult<()> {
        let value_start_point = start_point;
        TextRenderer::draw_text(&self.value_str, value_start_point, style, display)?;

        // if !self.const_already_drawn {
        //     let mut label_start_point = start_point;
        //     label_start_point.x += (VARIABLE_CHARS
        //         * (style.font.character_size.width as usize
        //             + style.font.character_spacing as usize))
        //         as i32; // value is 4 chars @ FONT_10X20
        //     TextRenderer::draw_text(&self.label_str, label_start_point, style, display)?;
        //     self.const_already_drawn = true;
        // }

        Ok(())
    }
}
