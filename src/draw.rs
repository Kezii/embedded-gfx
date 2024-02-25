use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::prelude::Point;

use crate::DrawPrimitive;

#[inline]
pub fn draw<D: DrawTarget<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    primitive: DrawPrimitive,
    fb: &mut D,
) where
    <D as DrawTarget>::Error: std::fmt::Debug,
{
    match primitive {
        DrawPrimitive::Line(p1, p2, color) => {
            fb.draw_iter(
                line_drawing::Bresenham::new((p1.x, p1.y), (p2.x, p2.y))
                    .map(|(x, y)| embedded_graphics_core::Pixel(Point::new(x, y), color)),
            )
            .unwrap();
        }
        DrawPrimitive::ColoredPoint(p, c) => {
            let p = embedded_graphics_core::geometry::Point::new(p.x, p.y);

            fb.draw_iter([embedded_graphics_core::Pixel(p, c)]).unwrap();
        }
        DrawPrimitive::ColoredTriangle(p1, p2, p3, color) => {
            //sort vertices by y
            let mut vertices = [p1, p2, p3];
            vertices.sort_by(|a, b| a.y.cmp(&b.y));

            let [p1, p2, p3] = vertices
                .iter()
                .map(|p| embedded_graphics_core::geometry::Point::new(p.x, p.y))
                .collect::<Vec<embedded_graphics_core::geometry::Point>>()
                .try_into()
                .unwrap();

            if p2.y == p3.y {
                fill_bottom_flat_triangle(p1, p2, p3, color, fb);
            } else if p1.y == p2.y {
                fill_top_flat_triangle(p1, p2, p3, color, fb);
            } else {
                let p4 = Point::new(
                    (p1.x as f32
                        + ((p2.y - p1.y) as f32 / (p3.y - p1.y) as f32) * (p3.x - p1.x) as f32)
                        as i32,
                    p2.y,
                );

                fill_bottom_flat_triangle(p1, p2, p4, color, fb);
                fill_top_flat_triangle(p2, p4, p3, color, fb);
            }
        }
    }
}

fn fill_bottom_flat_triangle<D: DrawTarget<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    p1: Point,
    p2: Point,
    p3: Point,
    color: embedded_graphics_core::pixelcolor::Rgb565,
    fb: &mut D,
) where
    <D as DrawTarget>::Error: std::fmt::Debug,
{
    let invslope1 = (p2.x - p1.x) as f32 / (p2.y - p1.y) as f32;
    let invslope2 = (p3.x - p1.x) as f32 / (p3.y - p1.y) as f32;

    let mut curx1 = p1.x as f32;
    let mut curx2 = p1.x as f32;

    for scanline_y in p1.y..=p2.y {
        draw_horizontal_line(
            Point::new(curx1 as i32, scanline_y),
            Point::new(curx2 as i32, scanline_y),
            color,
            fb,
        );

        curx1 += invslope1;
        curx2 += invslope2;
    }
}

fn fill_top_flat_triangle<D: DrawTarget<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    p1: Point,
    p2: Point,
    p3: Point,
    color: embedded_graphics_core::pixelcolor::Rgb565,
    fb: &mut D,
) where
    <D as DrawTarget>::Error: std::fmt::Debug,
{
    let invslope1 = (p3.x - p1.x) as f32 / (p3.y - p1.y) as f32;
    let invslope2 = (p3.x - p2.x) as f32 / (p3.y - p2.y) as f32;

    let mut curx1 = p3.x as f32;
    let mut curx2 = p3.x as f32;

    for scanline_y in (p1.y..=p3.y).rev() {
        draw_horizontal_line(
            Point::new(curx1 as i32, scanline_y),
            Point::new(curx2 as i32, scanline_y),
            color,
            fb,
        );

        curx1 -= invslope1;
        curx2 -= invslope2;
    }
}

fn draw_horizontal_line<D: DrawTarget<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    p1: Point,
    p2: Point,
    color: embedded_graphics_core::pixelcolor::Rgb565,
    fb: &mut D,
) where
    <D as DrawTarget>::Error: std::fmt::Debug,
{
    let start = p1.x.min(p2.x);
    let end = p1.x.max(p2.x);

    for x in start..=end {
        fb.draw_iter([embedded_graphics_core::Pixel(Point::new(x, p1.y), color)])
            .unwrap();
    }
}
