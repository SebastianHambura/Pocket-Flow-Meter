use embedded_charts::prelude::*;

pub struct History<const N: usize> {
    data: SlidingWindowSeries<Point2D, N>,
    x_axis_unit: &'static str,
    y_axis_unit: &'static str,
    label: Option<String<16>>,
}

impl<const N: usize> History<N> {
    pub fn new(
        x_axis_unit: &'static str,
        y_axis_unit: &'static str,
        label: Option<String<16>>,
    ) -> Self {
        Self {
            data: SlidingWindowSeries::new(),
            x_axis_unit,
            y_axis_unit,
            label,
        }
    }

    pub fn push(&mut self, point: Point2D) {
        self.data.push(point);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_newest(&self) -> Option<Point2D> {
        self.data.iter_chronological().last()
    }

    pub fn clone_latest_data<const M: usize>(&self) -> StaticDataSeries<Point2D, M> {
        let mut series = StaticDataSeries::<Point2D, M>::new();
        let iter = self.data.iter_chronological();
        let iter_len = self.data.current_len();
        let skipped = if iter_len > M { iter_len - M } else { 0 };

        for point in iter.skip(skipped) {
            series.push(point);
        }

        if let Some(label) = &self.label {
            series.set_label(label);
        }
        series
    }
}
