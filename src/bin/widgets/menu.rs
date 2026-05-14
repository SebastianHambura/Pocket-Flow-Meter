use embedded_graphics::{mono_font::MonoTextStyle, prelude::*};
use embedded_iconoir::size18px;

pub struct Menu<'a, C: PixelColor> {
    position: Point,

    left_arrow: crate::widgets::icons::DrawableIcon<C, size18px::navigation::NavArrowLeft>,
    right_arrow: crate::widgets::icons::DrawableIcon<C, size18px::navigation::NavArrowRight>,
    text_widget: crate::widgets::text::DrawableText<'a, C>,
}

impl<'a, C: PixelColor> Menu<'a, C> {
    pub fn new(
        color: C,
        position: Point,
        current_menu: Option<&'a str>,
        text_style: MonoTextStyle<'static, C>,
    ) -> Self {
        let margin = 3;
        let left_arrow =
            crate::widgets::icons::DrawableIcon::new(color, position + Point::new(0, 3));

        let text_widget = crate::widgets::text::DrawableText::new(
            position + Point::new(18 + margin, 0),
            current_menu.unwrap_or( "N/A."),
            text_style,
        );

        let right_arrow = crate::widgets::icons::DrawableIcon::new(
            color,
            position + Point::new(18 + margin + 4 * 10 + margin, 3),
        ); // TODO: calculate this based on the screen width

        Self {
            position,
            left_arrow,
            right_arrow,
            text_widget,
        }
    }

    pub fn update_menu(&mut self, new_menu: &'a str) {
        self.text_widget.update_text(new_menu);
    }
}

impl<'a, C: PixelColor> Drawable for Menu<'a, C> {
    type Color = C;

    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.left_arrow.draw(target)?;
        self.text_widget.draw(target)?;
        self.right_arrow.draw(target)?;
        Ok(())
    }
}
