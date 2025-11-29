use core::fmt::Write;

use embedded_charts::data::{OverflowMode, RingBufferConfig};
use embedded_charts::prelude::*;
use embedded_charts::{
    chart::AnimatedLineChart,
    data::{Point2D, PointRingBuffer, RingBuffer, RingBufferEvent, StaticDataSeries},
};
use embedded_graphics::mono_font::ascii::*;
use embedded_graphics::mono_font::MonoTextStyle;
use micromath::F32Ext;

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
}

impl<const N: usize> MeasuredValueWidget<N> {
    pub fn new(color: Rgb565) -> Self {
        let chart = AnimatedLineChart::builder()
            .line_color(color)
            .line_width(2)
            .with_markers(MarkerStyle {
                shape: MarkerShape::Circle,
                size: 4,
                color,
                visible: true,
            })
            .with_grid(
                GridSystem::builder()
                    .enabled(true)
                    .horizontal_linear(GridSpacing::Auto)
                    .vertical_linear(GridSpacing::Auto)
                    .build(),
            )
            .build()
            .unwrap();

        let stream = StreamingAnimator::<Point2D>::new();

        Self {
            stream,
            serie: StaticDataSeries::<Point2D, 256>::new(),
            chart,
        }
    }

    pub fn push_point(&mut self, point: Point2D) {
        //self.stream.update_with_delta(16) ;
        self.stream.push_data(point)
    }

    pub fn set_background_color(&mut self, background_color: Option<Rgb565>) {
        let mut config = self.chart.config().clone();
        config.background_color = background_color;
        self.chart.set_config(config);
    }

    pub fn draw_chart<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        viewport: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) -> ChartResult<()> {
        self.serie.clear();
        for point in self.stream.current_data() {
            self.serie.push(point);
        }

        self.chart
            .draw(&self.serie, &self.chart.config(), viewport, display)
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
        let mut flow = MeasuredValueWidget::new(Rgb565::CSS_STEEL_BLUE);
        flow.set_background_color(Some(Rgb565::WHITE));
        let flow_text = ValueWithLabelWidget::new("uL/min");

        // Temperature measurements
        let legend = legend
            .add_line_entry("Temperature", Rgb565::CSS_RED)
            .unwrap();
        let temp = MeasuredValueWidget::new(Rgb565::CSS_RED);
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
        let mut style = MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK);
        style.background_color = Some(Rgb565::WHITE);

        let mut point = Point {
            x: res.top_left.x,
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
