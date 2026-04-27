use core::any::{Any, TypeId};

use embedded_charts::prelude::*;
use kolibri_embedded_gui::ui::{GuiError, GuiResult, Response, Ui, Widget};

use crate::widgets::chart::ChartDrawable;

pub mod chart;
pub mod value;

impl<const N: usize> Widget for chart::StreamedDataPlot<N> {
    fn draw<DRAW, COL>(&mut self, ui: &mut Ui<DRAW, COL>) -> GuiResult<Response>
    where
        DRAW: DrawTarget<Color = COL>,
        COL: PixelColor,
    {
        // Based on the implementation for the Icons:
        let iresponse = ui.allocate_space(self.viewport.size)?;

        //pull colors from ui.style or something
        let style = ui.style();
        let colours = style_to_rgb565(style);

        let chart = self.create_chart(colours.0, Some(colours.1));

        //skip any smartstate stuff, always redraw
        ui.start_drawing(&iresponse.area);
        if !ui.cleared() {
            ui.clear_area(iresponse.area)?;
        }

        ui.draw(&chart)
            .map_err(|_| GuiError::DrawError(Some("Couldn't draw text")))?;

        ui.finalize()?;
        Ok(Response::new(iresponse))
    }
}

fn style_to_rgb565<COL: PixelColor + 'static>(
    style: &kolibri_embedded_gui::style::Style<COL>,
) -> (Rgb565, Rgb565) {
    let background_color = style.background_color;
    let line_color = style.primary_color;

    let background = <dyn Any>::downcast_ref::<Rgb565>(&background_color).copied();
    let line_color = <dyn Any>::downcast_ref::<Rgb565>(&line_color).copied();

    (
        background.unwrap_or(Rgb565::WHITE),
        line_color.unwrap_or(Rgb565::BLACK),
    )
}
