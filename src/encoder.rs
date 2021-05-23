use std::{fmt::Display, fs::File};

use bitvec::prelude::*;
use image::{DynamicImage, EncodableLayout, GenericImageView, Pixel};

use crate::prelude::{ImageIntrinsics, Image, Rgb, RgbChannel};

#[derive(Debug)]
pub struct ColorChange(u32, u32, Rgb<u8>, Rgb<u8>);

impl Display for ColorChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{} from {:?} to {:?}", self.0, self.1, self.2, self.3)
    }
}

#[derive(Debug)]
pub struct EncodeMap {
    pub encoded_byte: u8,
    pub affected_points: Vec<ColorChange>,
}

impl EncodeMap {
    pub fn new() -> Self {
        Self {
            encoded_byte: 0,
            affected_points: vec![],
        }
    }

    pub fn len(&self) -> usize {
        self.affected_points.len()
    }
}

#[derive(Debug)]
pub struct EncodedImage {
    altered_image: image::DynamicImage,
    original_image: image::DynamicImage,
    map: Vec<EncodeMap>,
}

impl EncodedImage {
    pub fn changes(&self) -> &Vec<EncodeMap> {
        &self.map
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let target_dimensions = self.altered_image.dimensions();
        match image::save_buffer(
            path,
            self.altered_image.as_bytes(),
            target_dimensions.0,
            target_dimensions.1,
            image::ColorType::Rgb8,
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct ImageEncoder {
    lsb_c: usize,
    skip_c: usize,
    offset: usize,
    spread: bool,
    encoding_channel: RgbChannel,
    source_image: DynamicImage,
}

impl Default for ImageEncoder {
    fn default() -> Self {
        Self {
            lsb_c: 1,
            skip_c: 1,
            offset: 0,
            spread: false,
            encoding_channel: RgbChannel::Blue,
            source_image: DynamicImage::new_rgb8(16, 16),
        }
    }
}

impl From<&str> for ImageEncoder {
    fn from(path: &str) -> Self {
        let mut file = File::open(path).expect("Test image not found");
        Self::from(&mut file as &mut dyn std::io::Read)
    }
}

impl From<&mut dyn std::io::Read> for ImageEncoder {
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

impl ImageEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encode_string(&self, data: String) -> Result<EncodedImage, String> {
        self.encode_data(data.as_bytes())
    }

    pub fn encode_bytes<'a>(&self, data: &'a [u8]) -> Result<EncodedImage, String> {
        self.encode_data(data.as_bytes())
    }

    pub fn encode_image(&self, image: &Image) -> Result<EncodedImage, String> {
        self.encode_data(image.to_rgb8().as_bytes())
    }

    fn encode_data<'a>(&self, data: &'a [u8]) -> Result<EncodedImage, String> {
        let img = &self.source_image;
        let mut encode_maps: Vec<EncodeMap> = vec![];

        if bytes_needed_for_data(data, self.lsb_c) <= img.as_bytes().len() {
            let mut rgb_img = img.to_rgb8();
            let mut pixel_iter = rgb_img
                .enumerate_pixels_mut()
                .skip(self.offset)
                .step_by(self.skip_c);
            for byte_to_encode in data.iter() {
                let mut current_byte_iter_count = 0;
                let mut current_byte_map = EncodeMap::new();
                current_byte_map.encoded_byte = byte_to_encode.clone();

                let bits_to_encode = byte_to_bits(byte_to_encode);

                if let Some(bits_ptr) = bits_to_encode {
                    // eprintln!("Encoding byte (lsb): {}", bits_ptr);
                    while current_byte_iter_count < std::mem::size_of::<u8>() * 8 {
                        let bits_to_encode_slice: &BitSlice<Lsb0, u8> = &bits_ptr
                            [current_byte_iter_count..current_byte_iter_count + self.lsb_c];
                        if let Some(pixel_to_modify) = pixel_iter.next() {
                            let mut color_change = ColorChange(
                                pixel_to_modify.0,
                                pixel_to_modify.1,
                                pixel_to_modify.2.clone().into(),
                                Rgb::from([0, 0, 0]),
                            );
                            let bits_to_modify = pixel_to_modify
                                .2
                                .channels_mut()
                                .get_mut::<usize>(self.encoding_channel.into())
                                .unwrap()
                                .view_bits_mut::<Lsb0>();
                            for i in 0..self.lsb_c {
                                bits_to_modify.set(i, bits_to_encode_slice[i]);
                            }

                            color_change.3 = pixel_to_modify.2.clone().into();
                            current_byte_map.affected_points.push(color_change);
                            current_byte_iter_count += self.lsb_c;
                        } else {
                            return Err(String::from(
                                "Not enough space in image to fit specified data",
                            ));
                        }
                    }
                }

                encode_maps.push(current_byte_map);
            }

            Ok(EncodedImage {
                original_image: img.clone(),
                altered_image: DynamicImage::ImageRgb8(rgb_img),
                map: encode_maps,
            })
        } else {
            Err(String::from(
                "Not enough space in image to fit specified data",
            ))
        }
    }
}

impl ImageIntrinsics for ImageEncoder {
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
    fn step_by_n_pixels(&mut self, n: usize) -> &mut Self {
        if n < 1 {
            self.skip_c = 1;
        } else {
            self.skip_c = n;
        }
        self
    }

    fn spread(&mut self, value: bool) -> &mut Self {
        self.spread = value;
        self
    }
}

fn bytes_needed_for_data(data: &[u8], using_n_lsb: usize) -> usize {
    (data.len() * 8) / using_n_lsb
}

fn byte_to_bits(byte: &u8) -> Option<&BitSlice<Lsb0, u8>> {
    let raw_bits = bitvec::ptr::bitslice_from_raw_parts::<Lsb0, u8>(
        BitPtr::from_ref(byte),
        8,
    );
    let bits;
    unsafe {
        bits = raw_bits.as_ref();
    }

    bits
}

#[allow(dead_code)]
fn eprint_color_changes(byte_map: &EncodeMap, steps: usize) {
    eprint!(
        "Encoded in {} steps, {} pixel(s) modified -> ",
        steps,
        byte_map.affected_points.len()
    );
    for item in &byte_map.affected_points {
        eprint!(" | {}", item);
    }
    eprintln!("\n\n");
}

#[cfg(test)]
mod test {
    fn ensure_out_dir() -> std::io::Result<()> {
        std::fs::create_dir_all("tests/out")
    }

    use crate::prelude::*;

    #[test]
    fn target_byte_size_calc() {
        assert_eq!(super::bytes_needed_for_data(&[8, 1, 2, 3], 1), 32);
        assert_eq!(super::bytes_needed_for_data(&[8, 1, 2, 3], 2), 16);
    }

    #[test]
    fn simple_encoding() {
        ensure_out_dir().unwrap();

        let encode_result = super::ImageEncoder::from("tests/images/red_panda.jpg")
            .use_n_lsb(2)
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
            .save("tests/out/red_panda_steg.jpeg")
            .expect("Could not create output file");
    }
}
