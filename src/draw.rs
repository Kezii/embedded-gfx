use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::prelude::Point;

use crate::canvas::{DrawError, GFX2DCanvas};
use crate::DrawPrimitive;

#[inline]
pub fn draw<D: GFX2DCanvas<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    primitive: &DrawPrimitive,
    fb: &mut D,
) -> Result<(), DrawError> {
    match *primitive {
        DrawPrimitive::Line([p1, p2], color) => {
            fb.draw_line(Point::new(p1.x, p1.y), Point::new(p2.x, p2.y), color)?;
        }
        DrawPrimitive::ColoredPoint(p, c) => {
            let p = embedded_graphics_core::geometry::Point::new(p.x, p.y);

            fb.draw_pixel(p, c)?;
        }
        DrawPrimitive::ColoredTriangle(mut vertices, color) => {
            //sort vertices by y
            vertices.sort_by(|a, b| a.y.cmp(&b.y));

            let [p1, p2, p3] = vertices
                .iter()
                .map(|p| embedded_graphics_core::geometry::Point::new(p.x, p.y))
                .collect::<Vec<embedded_graphics_core::geometry::Point>>()
                .try_into()
                .unwrap();

            if p2.y == p3.y {
                fill_bottom_flat_triangle(p1, p2, p3, color, fb)?;
            } else if p1.y == p2.y {
                fill_top_flat_triangle(p1, p2, p3, color, fb)?;
            } else {
                let p4 = Point::new(
                    (p1.x as f32
                        + ((p2.y - p1.y) as f32 / (p3.y - p1.y) as f32) * (p3.x - p1.x) as f32)
                        as i32,
                    p2.y,
                );

                fill_bottom_flat_triangle(p1, p2, p4, color, fb)?;
                fill_top_flat_triangle(p2, p4, p3, color, fb)?;
            }
        }
    }

    Ok(())
}

fn fill_bottom_flat_triangle<D: GFX2DCanvas<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    p1: Point,
    p2: Point,
    p3: Point,
    color: embedded_graphics_core::pixelcolor::Rgb565,
    fb: &mut D,
) -> Result<(), DrawError> {
    let invslope1 = (p2.x - p1.x) as f32 / (p2.y - p1.y) as f32;
    let invslope2 = (p3.x - p1.x) as f32 / (p3.y - p1.y) as f32;

    let mut curx1 = p1.x as f32;
    let mut curx2 = p1.x as f32;

    for scanline_y in p1.y..=p2.y {
        fb.draw_horizontal_line(
            Point::new(curx1 as i32, scanline_y),
            Point::new(curx2 as i32, scanline_y),
            color,
        )?;

        curx1 += invslope1;
        curx2 += invslope2;
    }

    Ok(())
}

fn fill_top_flat_triangle<D: GFX2DCanvas<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    p1: Point,
    p2: Point,
    p3: Point,
    color: embedded_graphics_core::pixelcolor::Rgb565,
    fb: &mut D,
) -> Result<(), DrawError> {
    let invslope1 = (p3.x - p1.x) as f32 / (p3.y - p1.y) as f32;
    let invslope2 = (p3.x - p2.x) as f32 / (p3.y - p2.y) as f32;

    let mut curx1 = p3.x as f32;
    let mut curx2 = p3.x as f32;

    for scanline_y in (p1.y..=p3.y).rev() {
        fb.draw_horizontal_line(
            Point::new(curx1 as i32, scanline_y),
            Point::new(curx2 as i32, scanline_y),
            color,
        )?;

        curx1 -= invslope1;
        curx2 -= invslope2;
    }

    Ok(())
}
