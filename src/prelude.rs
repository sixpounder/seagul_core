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
pub trait Encoder {
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
}
