use std::borrow::Cow;

use bitvec::{order::Lsb0, view::BitView};

use crate::prelude::{Encoder, RgbChannel};

pub struct DecodedImage {
    data: Vec<u8>,
    hit_marker: bool,
}

impl DecodedImage {
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

pub struct JpegDecoder {
    lsb_c: usize,
    skip_c: usize,
    encoding_channel: RgbChannel,
    offset: usize,
    marker: &'static [u8],
}

impl JpegDecoder {
    pub fn new() -> Self {
        Self {
            lsb_c: 1,
            skip_c: 1,
            offset: 0,
            encoding_channel: RgbChannel::Blue,
            marker: &[],
        }
    }

    /// Specifies a byte sequence to look for and stop deconding when found.
    pub fn until_marker(&mut self, marker_sequence: &'static [u8]) -> &mut Self {
        self.marker = marker_sequence;
        self
    }

    pub fn decode_buffer(&self, buf: &[u8]) -> Result<DecodedImage, String> {
        let byte_step = std::mem::size_of::<u8>() * 8;
        let mut decoded: Vec<u8> = Vec::with_capacity(100);
        let mut hit_marker = false;
        let target_sequence_len = self.marker.len();
        if let Ok(img) = image::load_from_memory(buf) {
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
                let pixel_lsb = pixel.2[self.encoding_channel.into()].view_bits::<Lsb0>();

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
                            if sequence_hint.as_slice() == self.marker {
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

            Ok(DecodedImage {
                data: decoded,
                hit_marker,
            })
        } else {
            Err(String::from("Could not decode image"))
        }
    }
}

impl Encoder for JpegDecoder {
    /// Skip the first `offset` bytes in the source buffer
    fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Sets the number of least significative bits to read for each
    /// byte in the source buffer. The default is 1.
    fn use_n_lsb(&mut self, n: usize) -> &mut Self {
        self.lsb_c = n;
        self
    }

    /// Specifies wich color channel will be the one used to store information bits.
    fn use_channel(&mut self, channel: RgbChannel) -> &mut Self {
        self.encoding_channel = channel;
        self
    }

    /// When decoding data, `n` pixels will be skipped after each edited pixel
    fn step_by_n_pixels(&mut self, n: usize) -> &mut Self {
        if n < 1 {
            self.skip_c = 1;
        } else {
            self.skip_c = n;
        }
        self
    }
}
