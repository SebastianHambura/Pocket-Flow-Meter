use anyhow::Context;
use embedded_charts::{
    chart::AnimatedLineChart,
    data::{Point2D, StaticDataSeries},
};
use embedded_charts::{grid, prelude::*};
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::mono_font::MonoTextStyle;

use crate::custom_widgets::ValueWithLabelWidget;
pub struct SensorWidget<const N: usize> {
    // The different graphs
    flow_graph: MeasuredValueWidget<N>,
    temperature_graph: MeasuredValueWidget<N>,

    // The legend for the graph
    legend: StandardLegend<Rgb565>,
    legend_renderer: StandardLegendRenderer<Rgb565>,
    legend_already_drawn: bool,

    // Displaying the real-time value
    flow_text: ValueWithLabelWidget<5, 6>,
    temperature_text: ValueWithLabelWidget<5, 6>,
}

struct MeasuredValueWidget<const N: usize> {
    stream: StreamingAnimator<Point2D>,
    serie: StaticDataSeries<Point2D, 256>,
    //buffer: RingBuffer<Point2D, N>,
    pub chart: AnimatedLineChart<Rgb565>,

    color: Rgb565,
    plot_background: bool,
    y_axis_side: AxisPosition,
}

impl<const N: usize> MeasuredValueWidget<N> {
    pub fn new(color: Rgb565, background_color: Option<Rgb565>, y_axis_side: AxisPosition) -> Self {
        // Create X-axis (horizontal, bottom) with calculated range
        // let y_axis = LinearAxisBuilder::new(AxisOrientation::Vertical, AxisPosition::Left)
        //     .range(0.0, 300.0)
        //     .build().unwrap();

        // Create Y-axis (vertical, left) with calculated range
        // let y_axis = presets::professional_y_axis(y_min, y_max)
        //     .tick_count(y_tick_count)
        //     .show_grid(true)
        //     .build()?;

        let chart = AnimatedLineChart::builder()
            .line_color(color)
            .line_width(2)
            .margins(Margins::symmetric(30, 10))
            .with_markers(MarkerStyle {
                shape: MarkerShape::Circle,
                size: 4,
                color,
                visible: true,
            });
        let chart = if let Some(color) = background_color {
            chart.background_color(color)
        } else {
            chart
        };
        let chart = chart.build().unwrap();

        let stream = StreamingAnimator::<Point2D>::new();

        Self {
            stream,
            serie: StaticDataSeries::<Point2D, 256>::new(),
            chart,
            color,
            plot_background: background_color.is_some(),
            y_axis_side,
        }
    }

    pub fn push_point(&mut self, point: Point2D) {
        //self.stream.update_with_delta(16) ;
        self.stream.push_data(point);
    }

    pub fn set_background_color(&mut self, background_color: Option<Rgb565>) {
        let mut config = self.chart.config().clone();
        config.background_color = background_color;
        self.chart.set_config(config);
    }

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

impl<const N: usize> SensorWidget<N> {
    pub fn new() -> Self {
        let legend = StandardLegendBuilder::new()
            .position(LegendPos::TopRight)
            .professional_style();

        // Flow measurments
        let legend = legend
            .add_line_entry("Flow", Rgb565::CSS_STEEL_BLUE)
            .unwrap();
        let mut flow = MeasuredValueWidget::new(
            Rgb565::CSS_STEEL_BLUE,
            Some(Rgb565::WHITE),
            AxisPosition::Left,
        );
        let flow_text = ValueWithLabelWidget::new("uL/min");

        // Temperature measurements
        let legend = legend.add_line_entry("Temp", Rgb565::CSS_RED).unwrap();
        let temp = MeasuredValueWidget::new(Rgb565::CSS_RED, None, AxisPosition::Right);
        let temp_text = ValueWithLabelWidget::new("°C");

        // Create legend
        let legend = legend.build().unwrap();
        let legend_renderer: StandardLegendRenderer<Rgb565> = StandardLegendRenderer::new();

        Self {
            flow_graph: flow,
            temperature_graph: temp,
            legend,
            legend_renderer,
            legend_already_drawn: false,
            flow_text,
            temperature_text: temp_text,
        }
    }

    pub fn new_sensor_value(&mut self, measurement: crate::sensor::Measurement) {
        self.flow_graph.push_point(measurement.flow);
        self.flow_text.update_value(measurement.flow.y);
        self.temperature_graph.push_point(measurement.temp);
        self.temperature_text.update_value(measurement.temp.y);
    }

    pub fn legend_widget<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        res: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) -> ChartResult<()> {
        if !self.legend_already_drawn {
            self.legend_renderer.render(&self.legend, res, display)?;
            self.legend_already_drawn = true;
        }
        Ok(())
    }

    pub fn current_values_widget<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        res: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) -> Result<(), RenderError> {
        let mut style = MonoTextStyle::new(&FONT_6X12, Rgb565::BLACK);
        style.background_color = Some(Rgb565::WHITE);

        let mut point = Point {
            x: res.top_left.x - 15, //We going a bit over the plot
            y: res.bottom_right().unwrap().y - style.font.character_size.height as i32,
        };
        self.flow_text.draw(point, &style, display)?;

        point.y -= style.font.character_size.height as i32;
        self.temperature_text.draw(point, &style, display)?;

        Ok(())
    }
}

impl SensorWidget<256> {
    // from embedded_charts: `type AnimatedData = StaticDataSeries<Point2D, 256>`
    pub fn chart<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        res: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) {
        // let flow_data = self.flow.get_static_data();
        // let temp_data = self.temperature.get_static_data();

        // let mut chart_config = self.flow.chart.config().clone();
        // chart_config.background_color = Some(Rgb565::CSS_AZURE);
        self.flow_graph.draw_chart(res, display).unwrap();
        self.temperature_graph.draw_chart(res, display).unwrap();
    }
}
