use embedded_graphics::{mono_font::MonoTextStyle, prelude::*, text::Text};

pub struct DrawableText<'a, C>
where
    C: PixelColor,
{
    // top_left corner of the text
    position: Point,
    text: &'a str,
    text_style: MonoTextStyle<'static, C>,
}

impl<'a, C> DrawableText<'a, C>
where
    C: PixelColor,
{
    pub fn new(position: Point, text: &'a str, text_style: MonoTextStyle<'static, C>) -> Self {
        let c_height = text_style.font.character_size.height as i32;
        Self {
            position: position + Point::new(0, c_height),
            text,
            text_style,
        }
    }

    pub fn update_text(&mut self, new_text: &'a str) {
        self.text = new_text;
    }
}

impl<'a, C> Drawable for DrawableText<'a, C>
where
    C: PixelColor,
{
    type Color = C;

    type Output = ();

    fn draw<T>(&self, target: &mut T) -> Result<(), <T>::Error>
    where
        T: DrawTarget<Color = C>,
    {    
        let text = Text::new(self.text, self.position , self.text_style);
        text.draw(target)?; 
        Ok(())
    }
}
