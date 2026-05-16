//! A custom chart widget.
//!
//! Based on embedded_charts

use embedded_charts::data::{Point2D, StaticDataSeries};
use embedded_charts::prelude::*;

pub type LineChartData = StaticDataSeries<Point2D, 256>;

pub struct StreamedDataPlot {
    viewport: embedded_graphics::primitives::Rectangle,

    // stream: StreamingAnimator<Point2D>,
    // serie: StaticDataSeries<Point2D, 256>,
    // //buffer: RingBuffer<Point2D, N>,
    data: LineChartData,
    chart: LineChart<Rgb565>,
    color: Rgb565,
    plot_background: bool,
    y_axis_side: AxisPosition,
}

impl StreamedDataPlot {
    pub fn create_chart(
        line_color: Rgb565,
        background_color: Option<Rgb565>,
    ) -> ChartResult<LineChart<Rgb565>> {
        let chart = LineChart::builder()
            .line_color(line_color)
            .line_width(2)
            .margins(Margins::new(0, 10, 10, 25)) // Axis graduation are not part of the plot ( are counted outside)
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
        viewport: embedded_graphics::primitives::Rectangle,
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
        Self {
            plot_background: background_color.is_some(),
            y_axis_side,
            chart,
            color: line_color,
            viewport,
            data: LineChartData::new(),
        }
    }

    // pub fn set_background_color(&mut self, background_color: Option<Rgb565>) {
    //     let mut config = self.chart.config().clone();
    //     config.background_color = background_color;
    //     self.chart.set_config(config);
    // }

    fn configure_axis(&mut self, bounds: DataBounds<f32, f32>) -> Result<(), ChartError> {
        let x_axis = presets::professional_x_axis(bounds.min_x, bounds.max_x)
            .show_grid(self.plot_background)
            .show_labels(self.plot_background)
            .show_line(self.plot_background)
            .show_ticks(self.plot_background)
            .build()?;
        self.chart.set_x_axis(x_axis);

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
            .build()?;
        self.chart.set_y_axis(y_axis);

        Ok(())
    }

    pub fn update_data<const N: usize>(&mut self, new_data: &crate::history::History<N>) {
        self.data = new_data.clone_latest_data();

        let bounds = self.data.bounds().unwrap_or(DataBounds {
            min_x: -1.0,
            max_x: 1.0,
            min_y: -1.0,
            max_y: 1.0,
        });
        if let Err(err) = self.configure_axis(bounds) {
            log::warn!("Failed to configure chart axis: {}", err);
        };
    }

    pub fn draw_chart<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        display: &mut T,
    ) -> ChartResult<()> {
        self.chart
            .draw(&self.data, self.chart.config(), self.viewport, display)
    }
}
