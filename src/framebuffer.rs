use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    pixelcolor::{IntoStorage, Rgb565},
};

pub struct DmaReadyFramebuffer<const W: usize, const H: usize, const BIG_ENDIAN: bool> {
    pub framebuffer: *mut [[u16; W]; H], // tfw no generic_const_exprs
}

impl<const W: usize, const H: usize, const BIG_ENDIAN: bool> DmaReadyFramebuffer<W, H, BIG_ENDIAN> {
    pub fn new(raw_framebuffer: *mut ::core::ffi::c_void) -> DmaReadyFramebuffer<W, H, BIG_ENDIAN> {
        if raw_framebuffer.is_null() {
            panic!("Failed to allocate framebuffer");
        }

        DmaReadyFramebuffer {
            framebuffer: raw_framebuffer as *mut [[u16; W]; H],
        }
    }

    pub fn set_pixel(&mut self, point: Point, color: Rgb565) {
        if point.x >= 0 && point.x < W as i32 && point.y >= 0 && point.y < H as i32 {
            unsafe {
                let framebuffer = &mut *self.framebuffer;

                match BIG_ENDIAN {
                    false => {
                        framebuffer[point.y as usize][point.x as usize] = color.into_storage();
                    }
                    true => {
                        framebuffer[point.y as usize][point.x as usize] =
                            color.into_storage().to_be();
                    }
                }
            }
        }
    }

    pub fn zero(&mut self) -> Result<(), core::convert::Infallible> {
        self.as_mut_slice().fill(0);
        Ok(())
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

impl<const W: usize, const H: usize, const BIG_ENDIAN: bool> DrawTarget
    for DmaReadyFramebuffer<W, H, BIG_ENDIAN>
{
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
        self.as_mut_slice().fill(color.into_storage());

        Ok(())
    }
}

impl<const W: usize, const H: usize, const BIG_ENDIAN: bool> OriginDimensions
    for DmaReadyFramebuffer<W, H, BIG_ENDIAN>
{
    fn size(&self) -> embedded_graphics_core::geometry::Size {
        embedded_graphics_core::geometry::Size::new(W as u32, H as u32)
    }
}
