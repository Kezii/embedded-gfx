use embedded_graphics_core::{
    pixelcolor::Rgb565,
    prelude::{PixelColor, Point},
};

use crate::framebuffer::RawFramebuffer;

#[derive(Debug)]
pub enum DrawError {
    OutOfBounds,
}

pub trait GFX2DCanvas: RawFramebuffer {
    type Color: PixelColor;

    fn draw_pixel(&mut self, point: Point, color: Rgb565) -> Result<(), DrawError> {
        if self.set_pixel(point, color) {
            Ok(())
        } else {
            Err(DrawError::OutOfBounds)
        }
    }

    fn draw_line(&mut self, p1: Point, p2: Point, color: Rgb565) -> Result<(), DrawError> {
        if p1.x < 0 && p2.x < 0 {
            return Ok(());
        }
        if p1.x >= self.limit().x && p2.x >= self.limit().x {
            return Ok(());
        }
        if p1.y < 0 && p2.y < 0 {
            return Ok(());
        }
        if p1.y >= self.limit().y && p2.y >= self.limit().y {
            return Ok(());
        }

        // fast path, unchecked
        if self.is_in_bounds(&p1) && self.is_in_bounds(&p2) {
            line_drawing::Bresenham::new((p1.x, p1.y), (p2.x, p2.y))
                .for_each(|(x, y)| self.set_pixel_unchecked(Point::new(x, y), color));

            return Ok(());
        }

        let errs = line_drawing::Bresenham::new((p1.x, p1.y), (p2.x, p2.y))
            .map(|(x, y)| self.set_pixel(Point::new(x, y), color))
            .filter(|e| !e)
            .last();

        if let Some(err) = errs {
            Err(DrawError::OutOfBounds)
        } else {
            Ok(())
        }
    }

    fn draw_horizontal_line(
        &mut self,
        p1: Point,
        p2: Point,
        color: Rgb565,
    ) -> Result<(), DrawError> {
        if p1.y < 0 || p1.y >= self.limit().y || p1.y != p2.y {
            return Err(DrawError::OutOfBounds);
        }

        let start = p1.x.min(p2.x);
        let end = p1.x.max(p2.x);

        let start = start.max(0).min(self.limit().x - 1);
        let end = end.max(0).min(self.limit().x - 1);

        for x in start..=end {
            self.set_pixel_unchecked(Point::new(x, p1.y), color);
        }

        Ok(())
    }
}
