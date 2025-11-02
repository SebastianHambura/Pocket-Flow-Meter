use embedded_charts::data::{OverflowMode, RingBufferConfig};
use embedded_charts::prelude::*;
use embedded_charts::{
    chart::AnimatedLineChart,
    data::{Point2D, PointRingBuffer, RingBuffer, RingBufferEvent, StaticDataSeries},
};
use micromath::F32Ext;

pub struct SensorWidget<const N: usize> {
    streaming_buffer: RingBuffer<Point2D, N>,
    chart: AnimatedLineChart<Rgb565>,
}

impl<const N: usize > SensorWidget<N> {
    pub fn new() -> Self {
        // Configure for real-time streaming
        let config = RingBufferConfig {
            overflow_mode: OverflowMode::Overwrite, // Overwrite oldest data
            enable_events: true,                    // Event notifications
            track_bounds: true,                     // Auto bounds tracking
            ..Default::default()
        };

        let mut streaming_buffer: RingBuffer<Point2D, N> = RingBuffer::with_config(config);
        // Set up event handler
        streaming_buffer.set_event_handler(|event| match event {
            RingBufferEvent::BufferFull => log::debug!("Buffer is now full!"),
            RingBufferEvent::BoundsChanged => log::debug!("Data bounds have changed"),
            _ => {}
        });

        let chart = AnimatedLineChart::builder()
            .line_color(Rgb565::BLUE)
            .line_width(2)
            .margins(Margins::symmetric(5, 5))
            .fill_area(Rgb565::BLACK) // Semi-transparent fill
            .frame_rate(1)
            .with_title("Test title")
            .with_grid(
                GridSystem::builder()
                    .enabled(true)
                    .horizontal_linear(GridSpacing::Auto)
                    .vertical_linear(GridSpacing::Auto)
                    .build(),
            )
            .build()
            .unwrap();

        Self {
            streaming_buffer: streaming_buffer,
            chart: chart,
        }
    }

    pub fn new_sensor_value(&mut self, measurement: crate::sensor::Measurement) {
        self.streaming_buffer
            .push_point(measurement.flow)
            .unwrap();
    }

    pub fn get_static_data(&self) -> StaticDataSeries<Point2D, 256> {
        // Use chronological iterator for proper time ordering
        let mut chart_data = StaticDataSeries::<Point2D, 256>::new();
        for point in self.streaming_buffer.iter_chronological() {
            chart_data.push(*point).unwrap();
        }
        chart_data
    }

    pub fn chart<T: DrawTarget<Color = Rgb565>>(
        &mut self,
        res: embedded_graphics::primitives::Rectangle,
        display: &mut T,
    ) {
        match self.chart.draw(
            &self.get_static_data(),
            self.chart.config(),
            res,
            display,
        ) {
            Ok(_) => (),
            Err(err) => {
                log::error!("{err:?}");
                ()
            }
        };
    }
}
