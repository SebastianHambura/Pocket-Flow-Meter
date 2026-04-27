//! A custom chart widget.
//!
//! Based on embedded_charts
use core::marker::PhantomData;

use anyhow::Context;
use embedded_bitmap_fonts::terminus::FONT_14x28_BOLD;
use embedded_charts::{
    chart::AnimatedLineChart,
    data::{Point2D, StaticDataSeries},
};
use embedded_charts::{grid, prelude::*};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::{
    mono_font::{ascii::*, iso_8859_15::FONT_10X20},
    text::Text,
};

pub struct StreamedDataPlot<const N: usize> {
    stream: StreamingAnimator<Point2D>,
    serie: StaticDataSeries<Point2D, 256>,
    //buffer: RingBuffer<Point2D, N>,
    pub chart: AnimatedLineChart<Rgb565>,
    color: Rgb565,
    plot_background: bool,
    y_axis_side: AxisPosition,
}

impl<const N: usize> StreamedDataPlot<N> {
    pub fn create_chart(
        line_color: Rgb565,
        background_color: Option<Rgb565>,
    ) -> ChartResult<AnimatedLineChart<Rgb565>> {
        let chart = AnimatedLineChart::builder()
            .line_color(line_color)
            .line_width(2)
            .margins(Margins::new(0, 20, 10, 10)) // Axis gradutation are not part of the plot ( are counted outside)
            .with_markers(MarkerStyle {
                shape: MarkerShape::Circle,
                size: 4,
                color: line_color,
                visible: true,
            });
        let chart = if let Some(color) = background_color {
            chart.background_color(color)
        } else {
            chart
        };
        chart.build()
    }

    pub fn new(
        line_color: Rgb565,
        background_color: Option<Rgb565>,
        y_axis_side: AxisPosition,
    ) -> Self {
        // Create X-axis (horizontal, bottom) with calculated range
        // let y_axis = LinearAxisBuilder::new(AxisOrientation::Vertical, AxisPosition::Left)
        //     .range(0.0, 300.0)
        //     .build().unwrap();

        // Create Y-axis (vertical, left) with calculated range
        // let y_axis = presets::professional_y_axis(y_min, y_max)
        //     .tick_count(y_tick_count)
        //     .show_grid(true)
        //     .build()?;
        let chart = Self::create_chart(line_color, background_color).unwrap();
        let stream = StreamingAnimator::<Point2D>::new();
        Self {
            stream,
            serie: StaticDataSeries::<Point2D, 256>::new(),
            plot_background: background_color.is_some(),
            y_axis_side,
            chart,
            color: line_color,
        }
    }

    pub fn push_point(&mut self, point: Point2D) {
        //self.stream.update_with_delta(16) ;
        self.stream.push_data(point);
    }

    // pub fn set_background_color(&mut self, background_color: Option<Rgb565>) {
    //     let mut config = self.chart.config().clone();
    //     config.background_color = background_color;
    //     self.chart.set_config(config);
    // }

    fn auto_range_axis(&mut self) -> anyhow::Result<()> {
        let bounds = self
            .serie
            .bounds()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("Retreiving bounds of Dataserie")?
            .nice_bounds();

        let x_axis = presets::professional_x_axis(bounds.min_x, bounds.max_x)
            .show_grid(self.plot_background)
            .show_labels(self.plot_background)
            .show_line(self.plot_background)
            .show_ticks(self.plot_background)
            .build()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("Building x axis")?;
        self.chart.base_chart_mut().set_x_axis(x_axis);

        let axis_style = AxisStyle::new()
            .with_axis_line(LineStyle::solid(self.color))
            .with_major_ticks(TickStyle::new(self.color, 8))
            .with_minor_ticks(
                TickStyle::new(
                    self.color, // Gray
                    4,          // Normal length
                )
                .with_width(1),
            )
            .with_grid_lines(LineStyle::solid(self.color))
            .with_labels(embedded_charts::axes::LabelStyle::new(self.color))
            .with_label_offset(10);

        let y_axis = LinearAxisBuilder::new(AxisOrientation::Vertical, self.y_axis_side)
            .range(bounds.min_y, bounds.max_y)
            .show_grid(self.y_axis_side != AxisPosition::Right)
            .with_minor_ticks(4)
            .style(axis_style)
            .build()
            .map_err(|err| anyhow::anyhow!("{}", err))
            .context("Building y axis")?;
        self.chart.base_chart_mut().set_y_axis(y_axis);

        Ok(())
    }

    pub fn draw_chart<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        viewport: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) -> ChartResult<()> {
        self.serie.clear();
        for point in self.stream.current_data() {
            let _ = self.serie.push(point);
        }

        if let Err(err) = self
            .auto_range_axis()
            .context("Trying to update the axis with the bounds of the datasets")
        {
            log::warn!("{:#}", err)
        };

        self.chart
            .draw(&self.serie, self.chart.config(), viewport, display)
    }
}
