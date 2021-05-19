use bitvec::prelude::*;
use image::{DynamicImage, Pixel};

use crate::prelude::{EncodingOptions, RgbChannel};

pub struct JpegEncoder {
    lsb_c: usize,
    skip_c: usize,
    offset: usize,
    encoding_channel: RgbChannel,
    source: Vec<u8>,
}

impl JpegEncoder {
    pub fn new() -> Self {
        Self {
            lsb_c: 1,
            skip_c: 0,
            offset: 0,
            encoding_channel: RgbChannel::Blue,
            source: vec![],
        }
    }

    pub fn source_data(&mut self, source_data: Vec<u8>) -> &mut Self {
        self.source = source_data.clone();
        self
    }

    pub fn encode_data(&self, data: &[u8]) -> Result<DynamicImage, String> {
        let mut s = self.source.clone();
        self.encode_buffer(&mut s, data)
    }

    fn encode_buffer(&self, buf: &[u8], data: &[u8]) -> Result<DynamicImage, String> {
        if let Ok(img) = image::load_from_memory(buf) {
            if bytes_need_for_data(data, self.lsb_c) <= img.as_bytes().len() {
                // let mut change_ops: usize = 0;
                let mut data_iter = data.iter();
                let mut rgb_img = img.to_rgb8();
                'pixels: for pixel in rgb_img.pixels_mut() {
                    let channel_opt = pixel
                        .channels_mut()
                        .get_mut::<usize>(self.encoding_channel.into());
                    if let Some(channel) = channel_opt {
                        let byte_to_modify = channel.view_bits_mut::<Lsb0>();
                        match data_iter.next() {
                            Some(byte_to_encode) => {
                                let raw_bits_to_encode;
                                raw_bits_to_encode = bitvec::ptr::bitslice_from_raw_parts::<Lsb0, u8>(
                                    BitPtr::from_ref(byte_to_encode),
                                    8,
                                );
                                let bits_to_encode;
                                unsafe {
                                    bits_to_encode = raw_bits_to_encode.as_ref();
                                }
                                if let Some(bits_ptr) = bits_to_encode {
                                    // change_ops += self.lsb_c;
                                    let bits_to_encode: &BitSlice<Lsb0, u8> = &bits_ptr[0..self.lsb_c];
                                    for i in 0..self.lsb_c {
                                        byte_to_modify.set(i, bits_to_encode[i]);
                                    }
                                }
                            }
                            None => break 'pixels,
                        }
                    } else {
                        return Err(String::from("Specified channel not found"));
                    }
                }
                Ok(DynamicImage::ImageRgb8(rgb_img))
            } else {
                Err(String::from(
                    "Not enough space in image to fit specified data",
                ))
            }
        } else {
            Err(String::from("Could not decode image"))
        }
    }
}

impl EncodingOptions for JpegEncoder {
    /// Skip the first `offset` bytes in the source buffer
    fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Sets the number of least significative bits to edit for each
    /// byte in the source buffer. The default is 1. The higher the value gets
    /// the least space is required to encode data into the source, but the resulting
    /// image will get noticeably different from the original
    fn use_n_lsb(&mut self, n: usize) -> &mut Self {
        self.lsb_c = n;
        self
    }

    /// Specifies wich color channel will be the one used to store information bits.
    fn use_channel(&mut self, channel: RgbChannel) -> &mut Self {
        self.encoding_channel = channel;
        self
    }

    /// When encoding data, `n` pixels will be skipped after each edited pixel
    fn skip_n_pixels(&mut self, n: usize) -> &mut Self {
        self.skip_c = n;
        self
    }
}

fn bytes_need_for_data(data: &[u8], using_n_lsb: usize) -> usize {
    (data.len() * 8) / using_n_lsb
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::{fs::File, io::Read};

    #[test]
    fn target_byte_size_calc() {
        assert_eq!(super::bytes_need_for_data(&[8, 1, 2, 3], 1), 32);
        assert_eq!(super::bytes_need_for_data(&[8, 1, 2, 3], 2), 16);
    }

    #[test]
    fn simple_encoding() {
        let mut file = File::open("tests/images/red_panda.jpg").expect("Test image not found");
        let mut source_data: Vec<u8> = Vec::new();
        file.read_to_end(&mut source_data)
            .expect("Cannot test image");

        let encode_result = super::JpegEncoder::new()
            .use_n_lsb(2)
            .source_data(source_data)
            .encode_data(
                b"
                Midway upon the journey of our life
                I found myself within a forest dark,
                For the straightforward pathway had been lost.
                Ah me! how hard a thing it is to say
                What was this forest savage, rough, and stern,
                Which in the very thought renews the fear.
                So bitter is it, death is little more;
                But of the good to treat, which there I found,
                Speak will I of the other things I saw there.
                I cannot well repeat how there I entered,
                So full was I of slumber at the moment
                In which I had abandoned the true way.",
            );

        assert!(encode_result.is_ok(), "Encoding failed");

        encode_result
            .unwrap()
            .save("tests/out/steg.jpeg")
            .expect("Could not create output file");
    }
}
