use std::{borrow::Cow, fs::File, time::Duration};

use bitvec::{order::Lsb0, view::BitView};
use image::{DynamicImage, EncodableLayout};

use crate::prelude::{Configurable, ImagePosition, RgbChannel};

pub struct DecodedImage {
    data: Vec<u8>,
    hit_marker: bool,
    elapsed: std::time::Duration
}

impl DecodedImage {
    pub fn decode_time(&self) -> &Duration {
        &self.elapsed
    }

    pub fn as_raw(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.data)
    }

    pub fn as_string(&self) -> String {
        String::from_utf8(self.data.clone()).unwrap()
    }

    pub fn embedded_data(&self) -> &Vec<u8> {
        &self.data
    }

    /// If this is true, decoding stopped by hitting a marker specified in the
    /// `JpegDecoder` configuration
    pub fn hit_marker(&self) -> bool {
        self.hit_marker
    }
}

pub struct ImageDecoder {
    lsb_c: usize,
    skip_c: usize,
    encoding_channel: RgbChannel,
    offset: usize,
    spread: bool,
    encoding_position: ImagePosition,
    marker: Option<&'static [u8]>,
    source_image: DynamicImage,
}

impl From<&str> for ImageDecoder {
    fn from(path: &str) -> Self {
        let mut file = File::open(path).expect("Test image not found");
        Self::from(&mut file as &mut dyn std::io::Read)
    }
}

impl From<&mut dyn std::io::Read> for ImageDecoder {
    fn from(readable: &mut dyn std::io::Read) -> Self {
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

impl From<&mut File> for ImageDecoder {
    fn from(source_file: &mut File) -> Self {
        Self::from(source_file as &mut dyn std::io::Read)
    }
}

impl Default for ImageDecoder {
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

impl ImageDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Specifies a byte sequence to look for and stop deconding when found.
    pub fn until_marker(&mut self, marker_sequence: Option<&'static [u8]>) -> &mut Self {
        self.marker = marker_sequence;
        self
    }

    pub fn decode(&self) -> Result<DecodedImage, String> {
        let start = std::time::Instant::now();
        let byte_step = std::mem::size_of::<u8>() * 8;
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

            // take lsb_c from this pixel channel
            for i in 0..self.lsb_c {
                current_byte_as_bits.set(iter_count, pixel_lsb[i]);
                iter_count += 1;
            }

            // Check if a single output byte is completed
            if iter_count == byte_step {
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
            elapsed: (end - start)
        })
    }
}

impl Configurable for ImageDecoder {
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

    /// When decoding data, `n` pixels will be skipped after each edited pixel
    fn set_step_by_n_pixels(&mut self, n: usize) -> &mut Self {
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
