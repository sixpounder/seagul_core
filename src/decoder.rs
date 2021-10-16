use std::{borrow::Cow, fs::File, string::FromUtf8Error, time::Duration};

use bitvec::{order::Lsb0, view::BitView};
use image::{DynamicImage, EncodableLayout};

use crate::prelude::{ImagePosition, ImageRules, RgbChannel};

const BYTE_STEP: usize = std::mem::size_of::<u8>() * 8;

pub struct DecodedImage {
    data: Vec<u8>,
    hit_marker: bool,
    elapsed: std::time::Duration,
}

impl DecodedImage {
    /// The time it took to decode the image
    pub fn decode_time(&self) -> &Duration {
        &self.elapsed
    }

    /// Decoded data as a raw string
    pub fn as_raw(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.data)
    }

    /// Tries to view the decoded data as valid Utf8
    pub fn as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    /// Gets a reference to the decoded byte array
    pub fn embedded_data(&self) -> &Vec<u8> {
        &self.data
    }

    /// If this is true, decoding stopped by hitting a marker specified in the
    /// `ImageDecoder` configuration
    pub fn hit_marker(&self) -> bool {
        self.hit_marker
    }

    /// Writes decoded bytes to a target `std::io::Write`
    pub fn write<W>(&self, w: &mut W) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        w.write_all(self.data.as_bytes())
    }
}

/// An image decoder tries to find data encoded into an image's pixels. Supports the same
/// configuration options as the `ImageEncoder`
#[derive(Debug)]
pub struct ImageDecoder<'a> {
    lsb_c: usize,
    skip_c: usize,
    encoding_channel: RgbChannel,
    offset: usize,
    spread: bool,
    encoding_position: ImagePosition,
    marker: Option<&'a [u8]>,
    source_image: DynamicImage,
}

impl<'a> From<&str> for ImageDecoder<'a> {
    fn from(path: &str) -> Self {
        let mut file = File::open(path).expect("Image not found");
        Self::from(&mut file as &mut dyn std::io::Read)
    }
}

impl<'a, R: std::io::Read + ?Sized> From<&mut R> for ImageDecoder<'a> {
    fn from(readable: &mut R) -> Self {
        let mut source_data: Vec<u8> = Vec::new();
        readable
            .read_to_end(&mut source_data)
            .expect("Cannot load image from this path");

        let img = image::load_from_memory(source_data.as_bytes()).unwrap();

        let mut this = Self::default();
        this.source_image = img;
        this
    }
}

impl<'a> Default for ImageDecoder<'a> {
    fn default() -> Self {
        Self {
            lsb_c: 1,
            skip_c: 1,
            offset: 0,
            spread: false,
            marker: None,
            encoding_position: ImagePosition::TopLeft,
            encoding_channel: RgbChannel::Blue,
            source_image: DynamicImage::new_rgb8(16, 16),
        }
    }
}

impl<'a> ImageDecoder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies a byte sequence to look for and stop deconding when found.
    pub fn until_marker(&mut self, marker_sequence: Option<&'a [u8]>) -> &mut Self {
        self.marker = marker_sequence;
        self
    }

    pub fn decode(&self) -> Result<DecodedImage, String> {
        let start = std::time::Instant::now();
        let decoding_channel = self.get_use_channel().into();
        let mut decoded: Vec<u8> = Vec::with_capacity(100);
        let mut hit_marker = false;
        let target_sequence = self.marker.unwrap_or(&[]);
        let target_sequence_len = target_sequence.len();
        let img = &self.source_image;
        let mut sequence_hint: Vec<u8> = Vec::with_capacity(target_sequence_len);
        let mut current_byte: u8 = 0b0000_0000;
        let mut current_byte_as_bits = current_byte.view_bits_mut::<Lsb0>();
        let mut iter_count: usize = 0;
        let rgb_img = img.to_rgb8();
        'pixel_iter: for pixel in rgb_img
            .enumerate_pixels()
            .skip(self.offset)
            .step_by(self.skip_c)
        {
            let pixel_lsb = pixel.2[decoding_channel].view_bits::<Lsb0>();

            // take lsb_c from this pixel target channel
            for i in 0..self.lsb_c {
                current_byte_as_bits.set(iter_count, pixel_lsb[i]);
                iter_count += 1;
            }

            // Check if a single output byte is completed
            if iter_count == BYTE_STEP {
                decoded.push(current_byte);
                if target_sequence_len != 0 {
                    sequence_hint.push(current_byte);

                    if sequence_hint.len() > target_sequence_len {
                        sequence_hint.remove(0);
                    }

                    if sequence_hint.len() == target_sequence_len {
                        if sequence_hint.as_slice() == target_sequence {
                            hit_marker = true;
                            break 'pixel_iter;
                        }
                    }
                }
                iter_count = 0;
                current_byte = 0b0000_0000;
                current_byte_as_bits = current_byte.view_bits_mut::<Lsb0>();
            }
        }

        let end = std::time::Instant::now();
        Ok(DecodedImage {
            data: decoded,
            hit_marker,
            elapsed: (end - start),
        })
    }
}

impl<'a> ImageRules for ImageDecoder<'_> {
    /// Skip the first `offset` bytes in the source buffer
    fn set_offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Sets the number of least significative bits to read for each
    /// byte in the source buffer. The default is 1.
    fn set_use_n_lsb(&mut self, n: usize) -> &mut Self {
        self.lsb_c = n;
        self
    }

    /// Specifies wich color channel will be the one used to store information bits.
    fn set_use_channel(&mut self, channel: RgbChannel) -> &mut Self {
        self.encoding_channel = channel;
        self
    }

    /// When decoding data, `n` pixels will be skipped after each edited pixel.
    /// If `n < 1` is passed, it defaults to `1`.
    fn set_step_by_n_pixels(&mut self, n: usize) -> &mut Self {
        // Not using `clamp` because we don't want to panic
        if n < 1 {
            self.skip_c = 1;
        } else {
            self.skip_c = n;
        }
        self
    }

    fn set_spread(&mut self, value: bool) -> &mut Self {
        self.spread = value;
        self
    }

    fn set_position(&mut self, value: ImagePosition) -> &mut Self {
        self.encoding_position = value;
        self
    }

    fn get_use_n_lsb(&self) -> usize {
        self.lsb_c
    }

    fn get_offset(&self) -> usize {
        self.offset
    }

    fn get_step_by_n_pixels(&self) -> usize {
        self.skip_c
    }

    fn get_use_channel(&self) -> &RgbChannel {
        &self.encoding_channel
    }

    fn get_spread(&self) -> bool {
        self.spread
    }

    fn get_position(&self) -> &ImagePosition {
        &self.encoding_position
    }
}
