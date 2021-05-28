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

#[derive(Debug, Clone)]
pub enum ImagePosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    At(u32, u32)
}

/// Describes an RGB color
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

/// Represents a color channel in a pixel
#[derive(Debug, Clone)]
pub enum RgbChannel {
    Red,
    Green,
    Blue
}

impl AsRef<RgbChannel> for RgbChannel {
    fn as_ref(&self) -> &RgbChannel {
        &self
    }
}

impl From<&str> for RgbChannel {
    fn from(repr: &str) -> Self {
        match repr {
            "red" | "r" => RgbChannel::Red,
            "green" | "g" => RgbChannel::Green,
            "blue" | "b" => RgbChannel::Blue,
            _ => RgbChannel::Blue
        }
    }
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

impl Into<usize> for &RgbChannel {
    fn into(self) -> usize {
        match self {
            RgbChannel::Red => { 0 }
            RgbChannel::Green => { 1 }
            RgbChannel::Blue => { 2 }
        }
    }
}

/// Encoding and decoding options specify how to interpret a set of bytes in an image
pub trait ImageRules {
    /// Sets the number of least significative bits to edit for each
    /// byte in the source buffer. The higher the value gets
    /// the least space is required to encode data into the source, but the resulting
    /// image will get noticeably different from the original
    fn set_use_n_lsb(&mut self, n: usize) -> &mut Self;

    /// Skip the first `offset` bytes in the source buffer
    fn set_offset(&mut self, offset: usize) -> &mut Self;

    /// When encoding data, `n` pixels will be skipped after each edited pixel
    fn set_step_by_n_pixels(&mut self, n: usize) -> &mut Self;

    /// Specifies wich color channel will be the one used to store information bits.
    fn set_use_channel(&mut self, channel: RgbChannel) -> &mut Self;

    /// If the message is spread across the image
    fn set_spread(&mut self, value: bool) -> &mut Self;

    /// Starting position for the encoding. Irrelevant if spread is true
    fn set_position(&mut self, value: ImagePosition) -> &mut Self;

    /// Sets the number of least significative bits to edit for each
    /// byte in the source buffer. The higher the value gets
    /// the least space is required to encode data into the source, but the resulting
    /// image will get noticeably different from the original
    fn get_use_n_lsb(&self) -> usize;

    /// Skip the first `offset` bytes in the source buffer
    fn get_offset(&self) -> usize;

    /// When encoding data, one pixdel each `n` pixels will be used to encode.
    ///
    /// For example: using `1` means skipping no pixels
    fn get_step_by_n_pixels(&self) -> usize;

    /// Specifies wich color channel will be the one used to store information bits.
    fn get_use_channel(&self) -> &RgbChannel;

    /// If the message is spread across the image
    fn get_spread(&self) -> bool;

    /// Starting position for the encoding. Irrelevant if spread is true
    fn get_position(&self) -> &ImagePosition;
}

/// Enumerates supported image formats
pub enum ImageFormat {
    Jpeg,
    Png,
    Bmp
}

impl From<image::ImageFormat> for ImageFormat {
    fn from(f: image::ImageFormat) -> Self {
        f.into()
    }
}
