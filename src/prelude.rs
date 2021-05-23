use std::ops::Deref;

use image::Primitive;

pub struct Image {
    inner: image::DynamicImage
}

impl Deref for Image {
    type Target = image::DynamicImage;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub enum ImagePosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    At(u32, u32)
}

#[derive(Debug)]
pub struct Rgb<T>(T, T, T);

impl<T: Primitive> From<image::Rgb<T>> for Rgb<T> {
    fn from(color: image::Rgb<T>) -> Self {
        let c = color.0;
        Rgb(c[0], c[1], c[2])
    }
}

impl<T: Primitive> From<[T; 3]> for Rgb<T> {
    fn from(color: [T; 3]) -> Self {
        Rgb(color[0], color[1], color[2])
    }
}

impl<T: Primitive> Into<image::Rgb<T>> for Rgb<T> {
    fn into(self) -> image::Rgb<T> {
        image::Rgb([self.0, self.1, self.2])
    }
}

#[derive(Clone, Copy)]
pub enum RgbChannel {
    Red,
    Green,
    Blue
}

impl Into<u8> for RgbChannel {
    fn into(self) -> u8 {
        match self {
            RgbChannel::Red => { 0 }
            RgbChannel::Green => { 1 }
            RgbChannel::Blue => { 2 }
        }
    }
}

impl Into<usize> for RgbChannel {
    fn into(self) -> usize {
        match self {
            RgbChannel::Red => { 0 }
            RgbChannel::Green => { 1 }
            RgbChannel::Blue => { 2 }
        }
    }
}

/// Encoding options specify how to interpret a set of bytes in an image
pub trait ImageIntrinsics {
    /// Sets the number of least significative bits to edit for each
    /// byte in the source buffer. The higher the value gets
    /// the least space is required to encode data into the source, but the resulting
    /// image will get noticeably different from the original
    fn use_n_lsb(&mut self, n: usize) -> &mut Self;

    /// Skip the first `offset` bytes in the source buffer
    fn offset(&mut self, offset: usize) -> &mut Self;

    /// When encoding data, `n` pixels will be skipped after each edited pixel
    fn step_by_n_pixels(&mut self, n: usize) -> &mut Self;

    /// Specifies wich color channel will be the one used to store information bits.
    fn use_channel(&mut self, channel: RgbChannel) -> &mut Self;

    fn spread(&mut self, value: bool) -> &mut Self;
}
