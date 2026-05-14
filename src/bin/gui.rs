use embedded_charts::prelude::*;
use embedded_graphics::{
    mono_font::{ascii, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::Rectangle,
};
use embedded_iconoir::size18px;

use crate::{
    style::UiStyle,
    widgets::icons::{BlinkingSwitchIcon, DrawableIcon},
};

pub struct Ui<'a, C: PixelColor> {
    // === Header widgets ===
    connection_status: crate::widgets::icons::DrawableIcon<C, size18px::actions::DoubleCheck>,
    sensor_name: crate::widgets::text::DrawableText<'a, C>,
    blinking_icon:
        crate::widgets::icons::BlinkingSwitchIcon<C, size18px::music::Play, size18px::music::Pause>,

    // === Plot widget ===
    chart: crate::widgets::chart::StreamedDataPlot,

    // === Footer widgets ===
    menu: crate::widgets::menu::Menu<'a, C>,
    value: crate::widgets::value::ValueIndicatorWidget<5, 10>,
}

impl<'a> Ui<'a, Rgb565> {
    pub fn new(screen: Rectangle, sensor_name: &'a str, style: UiStyle) -> Self {
        // === Create the header stuff ===
        let connection_status = DrawableIcon::new(style.foreground_color, Point::new(3, 3));

        let text_style = MonoTextStyle::new(&ascii::FONT_10X20, style.foreground_color);
        let sensor_name =
            crate::widgets::text::DrawableText::new(Point::new(20, 0), sensor_name, text_style);

        let point = Point::new(screen.size.width as i32 - 20, 3);
        let blinking_icon = BlinkingSwitchIcon::new(
            style.foreground_color,
            style.background_color,
            esp_hal::time::Duration::from_millis(500),
            point,
            Some(true),
        );

        // === Plot ===
        let chart_allocation = Rectangle::new(
            Point { x: 0, y: 20 + 10 },
            Size {
                width: screen.size.width,
                height: 95,
            },
        );
        let chart = crate::widgets::chart::StreamedDataPlot::new(
            style.plot_color,
            Some(style.background_color),
            AxisPosition::Left,
            chart_allocation,
        );

        // === Footer widgets ===
        let menu = crate::widgets::menu::Menu::new(
            style.foreground_color,
            chart_allocation.bottom_right().unwrap().y_axis(),
            None,
            text_style,
        );
        let value_widget = crate::widgets::value::ValueIndicatorWidget::new(
            chart_allocation.bottom_right().unwrap().y_axis() + Point::new(100, 0),
            "Value:",
            style.background_color,
            style.foreground_color,
        );

        Self {
            connection_status,
            blinking_icon,
            sensor_name,
            chart,
            menu,
            value: value_widget,
        }
    }

    // Called at every iteration of the main loop to update the UI state based on the current sensor state and menu
    pub fn tick_update(&mut self, update_plot: bool, current_menu: &'a str) {
        self.blinking_icon.update(update_plot);
        self.menu.update_menu(current_menu);
    }

    pub fn sensor_value_update(&mut self, flow_value: Option<f32>) {
        if let Err(err) = self.value.update_value(flow_value) {
            log::error!("Failed to update sensor value: {:?}", err);
        }
    }

    pub fn set_flow_unit(&mut self, flow_unit: &'a str) {
        self.value.set_label(flow_unit);
    }

    pub fn chart_update<const N: usize>(&mut self, new_data: &crate::history::History<N>) {
        log::info!("Updating chart with {} new data points", new_data.len());
        self.chart.update_data(new_data);
    }

    pub fn draw<D>(&mut self, display: &mut D) -> Result<(), <D>::Error>
    where
        D: DrawTarget<Color = Rgb565> + embedded_graphics::geometry::OriginDimensions,
    {
        self.connection_status.draw(display)?;
        self.sensor_name.draw(display)?;
        self.blinking_icon.draw(display)?;

        if let Err(err) = self.chart.draw_chart(display) {
            log::error!("Failed to draw chart: {:?}", err);
        }

        self.menu.draw(display)?;
        self.value.draw(display)?;
        Ok(())
    }
}
