use std::ops::Deref;

use image::Primitive;

pub struct Image {
    inner: image::DynamicImage,
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
    At(u32, u32),
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
    Blue,
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
            _ => RgbChannel::Blue,
        }
    }
}

impl From<RgbChannel> for u8 {
    fn from(val: RgbChannel) -> Self {
        match val {
            RgbChannel::Red => { 0 }
            RgbChannel::Green => { 1 }
            RgbChannel::Blue => { 2 }
        }
    }
}

impl From<RgbChannel> for usize {
    fn from(val: RgbChannel) -> Self {
        match val {
            RgbChannel::Red => { 0 }
            RgbChannel::Green => { 1 }
            RgbChannel::Blue => { 2 }
        }
    }
}

impl From<&RgbChannel> for usize {
    fn from(val: &RgbChannel) -> Self {
        match val {
            RgbChannel::Red => { 0 }
            RgbChannel::Green => { 1 }
            RgbChannel::Blue => { 2 }
        }
    }
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

#[derive(Debug, Clone)]
pub enum CompressionType {
    /// Default compression level
    Default,
    /// Fast, minimal compression
    Fast,
    /// High compression level
    Best,
    /// Huffman coding compression
    Huffman,
    /// Run-length encoding compression
    Rle,
}

impl From<image::png::CompressionType> for CompressionType {
    fn from(original_type: image::png::CompressionType) -> Self {
        match original_type {
            image::png::CompressionType::Default => Self::Default,
            image::png::CompressionType::Fast => Self::Fast,
            image::png::CompressionType::Best => Self::Best,
            image::png::CompressionType::Huffman => Self::Huffman,
            image::png::CompressionType::Rle => Self::Rle,
            _ => Self::Default
        }
    }
}

impl From<CompressionType> for image::png::CompressionType {
    fn from(val: CompressionType) -> Self {
        match val {
            CompressionType::Default => image::png::CompressionType::Default,
            CompressionType::Fast => image::png::CompressionType::Fast,
            CompressionType::Best => image::png::CompressionType::Best,
            CompressionType::Huffman => image::png::CompressionType::Huffman,
            CompressionType::Rle => image::png::CompressionType::Rle,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilterType {
    /// No processing done, best used for low bit depth greyscale or data with a
    /// low color count
    NoFilter,
    /// Filters based on previous pixel in the same scanline
    Sub,
    /// Filters based on the scanline above
    Up,
    /// Filters based on the average of left and right neighbor pixels
    Avg,
    /// Algorithm that takes into account the left, upper left, and above pixels
    Paeth,
}

impl From<image::png::FilterType> for FilterType {
    fn from(original_filter: image::png::FilterType) -> Self {
        match original_filter {
            image::png::FilterType::NoFilter => Self::NoFilter,
            image::png::FilterType::Sub => Self::Sub,
            image::png::FilterType::Up => Self::Up,
            image::png::FilterType::Avg => Self::Avg,
            image::png::FilterType::Paeth => Self::Paeth,
            _ => Self::NoFilter,
        }
    }
}

impl From<FilterType> for image::png::FilterType {
    fn from(val: FilterType) -> Self {
        match val {
            FilterType::NoFilter => image::png::FilterType::NoFilter,
            FilterType::Sub => image::png::FilterType::Sub,
            FilterType::Up => image::png::FilterType::Up,
            FilterType::Avg => image::png::FilterType::Avg,
            FilterType::Paeth => image::png::FilterType::Paeth,
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

    /// Sets a byte value to use for message padding across the image
    fn set_padding(&mut self, value: &str) -> &mut Self;

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
