use embedded_graphics_core::draw_target::DrawTarget;
use embedded_graphics_core::prelude::Point;

use crate::canvas::{DrawError, GFX2DCanvas};
use crate::DrawPrimitive;

const FP_SHIFT: i64 = 16;

#[derive(Clone, Copy)]
struct EdgeStepper {
    x: i64,
    step: i64,
}

impl EdgeStepper {
    fn new(start: Point, end: Point, y: i32) -> Self {
        let dy = (end.y - start.y) as i64;
        debug_assert!(dy > 0);

        let step = (((end.x - start.x) as i64) << FP_SHIFT) / dy;
        let x = ((start.x as i64) << FP_SHIFT) + step * (y - start.y) as i64;

        Self { x, step }
    }

    fn current_x(self) -> i32 {
        fixed_to_i32(self.x)
    }

    fn advance(&mut self) {
        self.x += self.step;
    }
}

#[inline]
fn fixed_to_i32(value: i64) -> i32 {
    if value >= 0 {
        (value >> FP_SHIFT) as i32
    } else {
        -((-value) >> FP_SHIFT) as i32
    }
}

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

            let p1 = Point::new(vertices[0].x, vertices[0].y);
            let p2 = Point::new(vertices[1].x, vertices[1].y);
            let p3 = Point::new(vertices[2].x, vertices[2].y);

            if p1.x < 0 && p2.x < 0 && p3.x < 0 {
                return Ok(());
            }
            if p1.x >= fb.limit().x as i32
                && p2.x >= fb.limit().x as i32
                && p3.x >= fb.limit().x as i32
            {
                return Ok(());
            }
            if p1.y < 0 && p2.y < 0 && p3.y < 0 {
                return Ok(());
            }

            if p1.y >= fb.limit().y as i32
                && p2.y >= fb.limit().y as i32
                && p3.y >= fb.limit().y as i32
            {
                return Ok(());
            }

            fill_triangle(p1, p2, p3, color, fb)?;
        }
    }

    Ok(())
}

fn fill_triangle<D: GFX2DCanvas<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    p1: Point,
    p2: Point,
    p3: Point,
    color: embedded_graphics_core::pixelcolor::Rgb565,
    fb: &mut D,
) -> Result<(), DrawError> {
    let area2 =
        (p2.x - p1.x) as i64 * (p3.y - p1.y) as i64 - (p2.y - p1.y) as i64 * (p3.x - p1.x) as i64;

    if area2 == 0 {
        return Ok(());
    }

    let min_y = p1.y.max(0);
    let max_y = p3.y.min(fb.limit().y - 1);

    if min_y > max_y {
        return Ok(());
    }

    let short_edge_on_left = area2 < 0;
    let top_end = if p2.y == p3.y { p2.y } else { p2.y - 1 };
    let top_start = min_y;
    let top_end = top_end.min(max_y);

    if p1.y != p2.y && top_start <= top_end {
        fill_section(
            p1,
            p3,
            p1,
            p2,
            top_start,
            top_end,
            short_edge_on_left,
            color,
            fb,
        )?;
    }

    let bottom_start = p2.y.max(min_y);
    if p2.y != p3.y && bottom_start <= max_y {
        fill_section(
            p1,
            p3,
            p2,
            p3,
            bottom_start,
            max_y,
            short_edge_on_left,
            color,
            fb,
        )?;
    }

    Ok(())
}

fn fill_section<D: GFX2DCanvas<Color = embedded_graphics_core::pixelcolor::Rgb565>>(
    long_start: Point,
    long_end: Point,
    short_start: Point,
    short_end: Point,
    y_start: i32,
    y_end: i32,
    short_edge_on_left: bool,
    color: embedded_graphics_core::pixelcolor::Rgb565,
    fb: &mut D,
) -> Result<(), DrawError> {
    let mut long_edge = EdgeStepper::new(long_start, long_end, y_start);
    let mut short_edge = EdgeStepper::new(short_start, short_end, y_start);

    for y in y_start..=y_end {
        let long_x = long_edge.current_x();
        let short_x = short_edge.current_x();
        let (left_x, right_x) = if short_edge_on_left {
            (short_x, long_x)
        } else {
            (long_x, short_x)
        };

        fb.draw_horizontal_line(Point::new(left_x, y), Point::new(right_x, y), color)
            .ok();

        long_edge.advance();
        short_edge.advance();
    }

    Ok(())
}
