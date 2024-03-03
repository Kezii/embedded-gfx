use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    pixelcolor::{IntoStorage, Rgb565},
};

pub struct DmaReadyFramebuffer<const W: usize, const H: usize> {
    pub framebuffer: *mut [[u16; W]; H], // tfw no generic_const_exprs
    big_endian: bool,
}

impl<const W: usize, const H: usize> DmaReadyFramebuffer<W, H> {
    pub fn new(
        raw_framebuffer: *mut ::core::ffi::c_void,
        big_endian: bool,
    ) -> DmaReadyFramebuffer<W, H> {
        if raw_framebuffer.is_null() {
            panic!("Failed to allocate framebuffer");
        }

        DmaReadyFramebuffer {
            framebuffer: raw_framebuffer as *mut [[u16; W]; H],
            big_endian,
        }
    }

    pub fn set_pixel(&mut self, point: Point, color: Rgb565) {
        if point.x >= 0 && point.x < W as i32 && point.y >= 0 && point.y < H as i32 {
            unsafe {
                let framebuffer = &mut *self.framebuffer;

                if self.big_endian {
                    framebuffer[point.y as usize][point.x as usize] = color.into_storage().to_be();
                } else {
                    framebuffer[point.y as usize][point.x as usize] = color.into_storage();
                }
            }
        }
    }

    pub fn as_slice(&self) -> &[u16] {
        unsafe { core::slice::from_raw_parts(self.framebuffer as *const u16, W * H) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        unsafe { core::slice::from_raw_parts_mut(self.framebuffer as *mut u16, W * H) }
    }

    pub fn as_mut_ptr(&mut self) -> *mut [u16] {
        self.as_slice() as *const [u16] as *mut [u16]
    }
}

impl<const W: usize, const H: usize> DrawTarget for DmaReadyFramebuffer<W, H> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::prelude::Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let embedded_graphics_core::prelude::Pixel(point, color) = pixel;

            self.set_pixel(point, color);
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        if self.big_endian {
            self.as_mut_slice().fill(color.into_storage().to_be());
        } else {
            self.as_mut_slice().fill(color.into_storage());
        }

        Ok(())
    }
}

impl<const W: usize, const H: usize> OriginDimensions for DmaReadyFramebuffer<W, H> {
    fn size(&self) -> embedded_graphics_core::geometry::Size {
        embedded_graphics_core::geometry::Size::new(W as u32, H as u32)
    }
}
