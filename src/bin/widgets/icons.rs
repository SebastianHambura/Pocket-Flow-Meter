use embedded_graphics::{image::Image, prelude::*, Drawable};
use embedded_iconoir::prelude::*;

pub struct DrawableIcon<C, T>
where
    C: PixelColor,
    T: embedded_iconoir::prelude::IconoirIcon,
{
    icon: embedded_iconoir::Icon<C, T>,
    position: Point,
}

impl<C, T> DrawableIcon<C, T>
where
    C: PixelColor,
    T: embedded_iconoir::prelude::IconoirIcon,
{
    pub fn new(color: C, position: Point) -> Self {
        let icon = T::new(color);
        Self { icon, position }
    }
}

impl<C, T> Drawable for DrawableIcon<C, T>
where
    C: PixelColor,
    T: embedded_iconoir::prelude::IconoirIcon,
{
    type Color = C;

    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), <D>::Error>
    where
        D: embedded_charts::prelude::DrawTarget<Color = C>,
    {
        let img = Image::new(&self.icon, self.position);
        img.draw(target)
    }
}

/// A blinking icon that can switch between two icons based on a boolean value.
pub struct BlinkingSwitchIcon<C, IconIfTrue, IconIfFalse>
where
    C: PixelColor,
    IconIfTrue: embedded_iconoir::prelude::IconoirIcon,
    IconIfFalse: embedded_iconoir::prelude::IconoirIcon,
{
    position: Point,

    switch_value: bool,
    icon_if_true: embedded_iconoir::Icon<C, IconIfTrue>,
    icon_if_false: embedded_iconoir::Icon<C, IconIfFalse>,

    // Stuff needed for the blinking behavior
    other_color: C,
    period: esp_hal::time::Duration,
    last_toggle: esp_hal::time::Instant,
}

impl<C, IconIfTrue, IconIfFalse> BlinkingSwitchIcon<C, IconIfTrue, IconIfFalse>
where
    C: PixelColor,
    IconIfTrue: embedded_iconoir::prelude::IconoirIcon,
    IconIfFalse: embedded_iconoir::prelude::IconoirIcon,
{
    pub fn new(
        color_on: C,
        color_off: C,
        period: esp_hal::time::Duration,
        position: Point,
        value: Option<bool>,
    ) -> Self {
        let icon_if_true = IconIfTrue::new(color_on);
        let icon_if_false = IconIfFalse::new(color_on);
        Self {
            position,
            switch_value: value.unwrap_or(true),
            icon_if_true,
            icon_if_false,
            other_color: color_off,
            period,
            last_toggle: esp_hal::time::Instant::now(),
        }
    }

    pub fn update(&mut self, value: bool) {
        self.switch_value = value;

        let elapsed = self.last_toggle.elapsed();
        if elapsed >= self.period {
            // Toggle the icon color
            let old_color = self.icon_if_true.get_color();
            self.icon_if_true.set_color(self.other_color);
            self.icon_if_false.set_color(self.other_color);

            self.other_color = old_color; // Swap colors for the next toggle
            self.last_toggle = esp_hal::time::Instant::now(); // Reset the timer
        }
    }
}

impl<C, IconIfTrue, IconIfFalse> Drawable for BlinkingSwitchIcon<C, IconIfTrue, IconIfFalse>
where
    C: PixelColor,
    IconIfTrue: embedded_iconoir::prelude::IconoirIcon,
    IconIfFalse: embedded_iconoir::prelude::IconoirIcon,
{
    type Color = C;

    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), <D>::Error>
    where
        D: embedded_charts::prelude::DrawTarget<Color = C>,
    {
        if self.switch_value {
            let img = Image::new(&self.icon_if_true, self.position);
            img.draw(target)
        } else {
            let img = Image::new(&self.icon_if_false, self.position);
            img.draw(target)
        }
    }
}
