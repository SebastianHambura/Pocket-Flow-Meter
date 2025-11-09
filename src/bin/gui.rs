use embedded_charts::data::{OverflowMode, RingBufferConfig};
use embedded_charts::prelude::*;
use embedded_charts::{
    chart::AnimatedLineChart,
    data::{Point2D, PointRingBuffer, RingBuffer, RingBufferEvent, StaticDataSeries},
};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::*;
use micromath::F32Ext;

pub struct SensorWidget<const N: usize> {
    flow_buffer: RingBuffer<Point2D, N>,
    temp_buffer: RingBuffer<Point2D, N>,
    chart: AnimatedLineChart<Rgb565>,
    legend: StandardLegend<Rgb565>,
    legend_renderer: StandardLegendRenderer<Rgb565>,
}

impl<const N: usize> SensorWidget<N> {
    pub fn new() -> Self {
        // Configure for real-time streaming
        let config = RingBufferConfig {
            overflow_mode: OverflowMode::Overwrite, // Overwrite oldest data
            enable_events: true,                    // Event notifications
            track_bounds: true,                     // Auto bounds tracking
            ..Default::default()
        };

        let mut flow_buffer: RingBuffer<Point2D, N> = RingBuffer::with_config(config);
        // Set up event handler
        flow_buffer.set_event_handler(|event| match event {
            RingBufferEvent::BufferFull => log::debug!("Buffer is now full!"),
            RingBufferEvent::BoundsChanged => log::debug!("Data bounds have changed"),
            _ => {}
        });

        let mut temp_buffer: RingBuffer<Point2D, N> = RingBuffer::with_config(config);
        temp_buffer.set_event_handler(|event| match event {
            RingBufferEvent::BufferFull => log::debug!("Buffer is now full!"),
            RingBufferEvent::BoundsChanged => log::debug!("Data bounds have changed"),
            _ => {}
        });

        let chart = AnimatedLineChart::builder()
            .line_color(Rgb565::BLUE)
            .line_width(2)
            .margins(Margins::symmetric(5, 5))
            //.fill_area(Rgb565::BLACK) // Semi-transparent fill
            //.frame_rate(1)
            .with_title("Test title")
            .with_grid(
                GridSystem::builder()
                    .enabled(true)
                    .horizontal_linear(GridSpacing::Auto)
                    .vertical_linear(GridSpacing::Auto)
                    .build(),
            )
            .background_color(Rgb565::CSS_AZURE)
            .build()
            .unwrap();

        // Create legend
        let legend = StandardLegendBuilder::new()
            .position(LegendPos::TopRight)
            .add_line_entry("Temperature", Rgb565::CSS_STEEL_BLUE)
            .unwrap()
            .add_line_entry("Flow", Rgb565::CSS_RED)
            .unwrap()
            .professional_style()
            .build()
            .unwrap();
        let legend_renderer: StandardLegendRenderer<Rgb565> = StandardLegendRenderer::new();

        Self {
            flow_buffer: flow_buffer,
            temp_buffer: temp_buffer,
            chart: chart,
            legend: legend,
            legend_renderer: legend_renderer,
        }
    }

    pub fn new_sensor_value(&mut self, measurement: crate::sensor::Measurement) {
        self.flow_buffer.push_point(measurement.flow).unwrap();
        self.temp_buffer.push_point(measurement.temp).unwrap();
    }

    pub fn get_static_data(&self) -> MultiSeries<Point2D, 2, 256> {
        let mut multi_series: MultiSeries<Point2D, 2, 256> = MultiSeries::new();

        // Use chronological iterator for proper time ordering
        let mut flow_data = StaticDataSeries::<Point2D, 256>::new();
        for point in self.flow_buffer.iter_chronological() {
            flow_data.push(*point).unwrap();
        }
        let mut temp_data = StaticDataSeries::<Point2D, 256>::new();
        for point in self.temp_buffer.iter_chronological() {
            temp_data.push(*point).unwrap();
        }

        multi_series.add_series(flow_data).unwrap();
        multi_series.add_series(temp_data).unwrap();
        multi_series
    }

    pub fn chart<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        res: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) {
        let data = self.get_static_data();
        self.chart
            .draw(
                data.get_series(0).unwrap(),
                self.chart.config(),
                res,
                display,
            )
            .unwrap();
    }

    pub fn legend_widget<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        res: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) {
        self.legend_renderer
            .render(&self.legend, res, display)
            .unwrap();

        TextRenderer::draw_text(
            "99uL/min",
            Point { x: res.top_left.x, y: res.bottom_right().unwrap().y - 30 },
            &MonoTextStyle::new(&FONT_10X20, Rgb565::BLACK),
            display,
        )
        .unwrap();
    }
}
