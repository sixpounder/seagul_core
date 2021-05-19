use bitvec::{order::Lsb0, view::BitView};

use crate::prelude::{EncodingOptions, RgbChannel};

pub struct Decoded {
    data: Vec<u8>,
    hit_marker: bool,
}

impl Decoded {
    pub fn data(&self) -> &Vec<u8> {
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
            skip_c: 0,
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

    pub fn decode_buffer(&self, buf: &[u8]) -> Result<Decoded, String> {
        let byte_step = 8 / self.lsb_c;
        let mut decoded: Vec<u8> = Vec::with_capacity(100);
        let mut hit_marker = false;
        let target_sequence_len = self.marker.len();
        if let Ok(img) = image::load_from_memory(buf) {
            let mut sequence_hint: Vec<u8> = Vec::with_capacity(target_sequence_len);
            let mut current_byte: u8 = 0b0000_0000;
            let mut iter_count: usize = 0;
            'pixel_iter: for pixel in img.to_rgb16().pixels().skip(self.offset) {
                let pixel_lsb = pixel[self.encoding_channel.into()].view_bits::<Lsb0>();
                let current_byte_as_bits = current_byte.view_bits_mut::<Lsb0>();
                for i in 0..self.lsb_c {
                    current_byte_as_bits.set(i, pixel_lsb[i]);
                }
                // Check if a single output byte is completed
                if iter_count == byte_step {
                    decoded.push(current_byte);
                    if target_sequence_len != 0 {
                        sequence_hint.push(current_byte);
                        if sequence_hint.len() == target_sequence_len {
                            if sequence_hint.as_slice() == self.marker {
                                hit_marker = true;
                                break 'pixel_iter;
                            } else {
                                sequence_hint.remove(0);
                            }
                        }
                    }
                    iter_count = 0;
                    current_byte = 0b0000_0000;
                } else {
                    iter_count += 1;
                }
            }

            Ok(Decoded {
                data: decoded,
                hit_marker,
            })
        } else {
            Err(String::from("Could not decode image"))
        }
    }
}

impl EncodingOptions for JpegDecoder {
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

    fn skip_n_pixels(&mut self, n: usize) -> &mut Self {
        self.skip_c = n;
        self
    }
}
