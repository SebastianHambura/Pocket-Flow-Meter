

pub mod color_converter {
    use embedded_graphics::prelude::* ;
    use embedded_graphics::pixelcolor::{Rgb565, BinaryColor} ;

/// Custom BinaryColor to Rgb565 adapter for embedded-graphics
pub struct BinaryToRgb565<'a, T> {
    target: &'a mut T,
    on_color: Rgb565,
    off_color: Option<Rgb565>, // None = transparent
}

impl<'a, T> BinaryToRgb565<'a, T> {
    pub fn new(target: &'a mut T, on: Rgb565, off: Option<Rgb565>) -> Self {
        Self {
            target,
            on_color: on,
            off_color: off,
        }
    }
}

impl<T> DrawTarget for BinaryToRgb565<'_, T>
where
    T: DrawTarget<Color = Rgb565> + embedded_graphics::geometry::OriginDimensions,
{
    type Color = BinaryColor;
    type Error = T::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.target
            .draw_iter(pixels.into_iter().filter_map(|Pixel(p, c)| match c {
                BinaryColor::On => Some(Pixel(p, self.on_color)),
                BinaryColor::Off => self.off_color.map(|c| Pixel(p, c)),
            }))
    }
}

impl<T> OriginDimensions for BinaryToRgb565<'_, T>
where
    T: OriginDimensions,
{
    fn size(&self) -> embedded_graphics::geometry::Size {
        self.target.size()
    }
}

}