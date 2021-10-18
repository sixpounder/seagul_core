use std::{fmt::Display, fs::File};

use bitvec::{prelude::*, view::AsBits};
use image::{DynamicImage, EncodableLayout, GenericImageView, Pixel};

use crate::{conversion::byte_to_bits, prelude::{CompressionType, FilterType, ImageFormat, ImagePosition, ImageRules, Rgb, RgbChannel}};

/// Describes a color change for a pixel at coordinates `(.0, .1)` from color `.2` to color `.3`
#[derive(Debug)]
pub struct ColorChange(u32, u32, Rgb<u8>, Rgb<u8>);

impl Display for ColorChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{} from {:?} to {:?}", self.0, self.1, self.2, self.3)
    }
}

/// Describes how a single byte is encoded
#[derive(Debug)]
pub struct ByteEncodeMap {
    pub encoded_byte: u8,
    pub affected_points: Vec<ColorChange>,
}

impl ByteEncodeMap {
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

/// Represents the result of an image encoded with `ImageEncoder` and offers saving methods
#[derive(Debug)]
pub struct EncodedImage {
    altered_image: image::DynamicImage,
    original_image: image::DynamicImage,
    map: Vec<ByteEncodeMap>,
}

impl EncodedImage {
    pub fn changes(&self) -> &Vec<ByteEncodeMap> {
        &self.map
    }

    pub fn pixels_changed(&self) -> usize {
        *&self.map.iter().fold(0, |acc, item| acc + item.len())
    }

    /// Writes decoded bytes into a new file at `path`, with the specified image format.
    /// If the file exists it is overwritten.
    pub fn save(&self, path: &str, format: ImageFormat) -> Result<(), std::io::Error> {
        let mut output_file = File::create(path).unwrap();
        self.write(&mut output_file, format)
    }

    /// Writes decoded bytes into an arbitraty `std::io::Write`, with the specified image format
    pub fn write<W>(&self, writable: &mut W, format: ImageFormat) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        ImageWriter::new(&self).write(writable, format)
    }
}

/// Writes an `EncodedImage`to a `Writable`
pub struct ImageWriter<'a> {
    image: &'a EncodedImage,
    compression_type: CompressionType,
    filter_type: FilterType
}

impl<'a> ImageWriter<'a> {
    pub fn new(image_ref: &'a EncodedImage) -> Self {
        Self {
            image: image_ref,
            compression_type: CompressionType::Default,
            filter_type: FilterType::NoFilter
        }
    }

    /// Writes decoded bytes into an arbitraty `std::io::Write`, with the specified image format
    pub fn write<W>(&self, writable: &mut W, format: ImageFormat) -> Result<(), std::io::Error>
    where
        W: std::io::Write,
    {
        let target_dimensions = self.image.altered_image.dimensions();
        let bytes = self.image.altered_image.as_bytes();

        match format {
            ImageFormat::Jpeg | ImageFormat::Png => {
                match image::ImageEncoder::write_image(
                    image::png::PngEncoder::new_with_quality(
                        writable,
                        self.compression_type.clone().into(),
                        self.filter_type.clone().into(),
                    ),
                    bytes,
                    target_dimensions.0,
                    target_dimensions.1,
                    image::ColorType::Rgb8,
                ) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Interrupted, e)),
                }
            }
            ImageFormat::Bmp => {
                // Box::new(image::bmp::BmpEncoder::new(&mut output_file))
                match image::ImageEncoder::write_image(
                    image::bmp::BmpEncoder::new(writable),
                    bytes,
                    target_dimensions.0,
                    target_dimensions.1,
                    image::ColorType::Rgb8,
                ) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Interrupted, e)),
                }
            }
        }
    }
}

/// An image decoder takes an image and alters its pixels to encode arbitrary data
pub struct ImageEncoder {
    // Number of least significant bits to modify on each byte
    lsb_c: usize,

    // Number of bytes to skip after each modified byte
    skip_c: usize,

    // Number of bytes to skip before starting encode
    offset: usize,

    // Wheter to spread the message across all image
    spread: bool,

    // Fill all non-modified bytes with a fixed chunk of data
    padding: Option<String>,

    // The color channel to use for encoding
    encoding_channel: RgbChannel,

    // The position on the image to start encoding from
    encoding_position: ImagePosition,

    // The source image to be modified
    source_image: DynamicImage,
}

impl Default for ImageEncoder {
    fn default() -> Self {
        Self {
            lsb_c: 1,
            skip_c: 1,
            offset: 0,
            spread: false,
            padding: None,
            encoding_channel: RgbChannel::Blue,
            encoding_position: ImagePosition::TopLeft,
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

impl<R: std::io::Read + ?Sized> From<&mut R> for ImageEncoder {
    fn from(readable: &mut R) -> Self {
        let mut source_data: Vec<u8> = Vec::new();
        readable
            .read_to_end(&mut source_data)
            .expect("Cannot load image from this path");

        let img = image::load_from_memory(source_data.as_bytes()).unwrap();

        let mut encoder = Self::default();
        encoder.source_image = img;

        encoder
    }
}

impl ImageEncoder {
    /// Encodes a string into the source image for this decoder
    pub fn encode_string(&self, data: String) -> Result<EncodedImage, String> {
        self.encode_data(data.as_bytes())
    }

    /// Encodes arbitrary bytes into the source image for this decoder
    pub fn encode_bytes<'a>(&self, data: &'a [u8]) -> Result<EncodedImage, String> {
        self.encode_data(data.as_bytes())
    }

    fn encode_data<'a>(&self, data: &'a [u8]) -> Result<EncodedImage, String> {
        let img = &self.source_image;
        let mut encode_maps: Vec<ByteEncodeMap> = vec![];
        let encoding_channel = self.get_use_channel().into();
        let bytes_per_round = bytes_needed_for_data(data, self);

        // Determine padding bits option
        let mut padding_bits = None;
        let unwrapped_padding_str;
        if self.padding.is_some() {
            unwrapped_padding_str = self.padding.as_ref().unwrap();
            padding_bits = Some((*unwrapped_padding_str).as_bits::<Lsb0>());
        }

        if bytes_per_round <= img.as_bytes().len() {
            let mut rgb_img = img.to_rgb8();
            let image_dimensions = rgb_img.dimensions();
            let mut real_offset: usize = 0;
            match self.encoding_position {
                ImagePosition::TopLeft => (),
                ImagePosition::TopRight => {
                    real_offset = image_dimensions.0 as usize;
                }
                ImagePosition::BottomLeft => {
                    real_offset = image_dimensions.1 as usize;
                }
                ImagePosition::BottomRight => {
                    real_offset = image_dimensions.0 as usize + image_dimensions.1 as usize
                }
                ImagePosition::Center => {
                    real_offset = (image_dimensions.0 as usize + image_dimensions.1 as usize) / 2
                }
                ImagePosition::At(w, h) => {
                    real_offset = (w * h) as usize;
                }
            }

            real_offset += self.offset;
            
            let mut pixel_iter = rgb_img
                .enumerate_pixels_mut()
                .skip(real_offset)
                .step_by(self.skip_c);

            // while real_offset > 0 {
            //     pixel_iter.next();
            //     if let Some(_padding_bits_value) = padding_bits {
            //         // TODO: put leading padding bits
            //     }
            //     real_offset -= 1;
            // }

            let mut pixel_iter_counter = img.pixels().count();

            'encode_rounds: loop {
                let data_iterator = data.iter();
                'data_iter: for byte_to_encode in data_iterator {
                    let mut current_byte_iter_count = 0;
                    let mut current_byte_map = ByteEncodeMap::new();
                    current_byte_map.encoded_byte = byte_to_encode.clone();

                    let bits_to_encode = byte_to_bits(byte_to_encode);

                    if let Some(bits_ptr) = bits_to_encode {
                        while current_byte_iter_count < std::mem::size_of::<u8>() * 8 {

                            // Get the chunk of bits of lsb_c length at current_byte_iter_count offset
                            let bits_to_encode_slice: &BitSlice<Lsb0, u8> = &bits_ptr
                                [current_byte_iter_count..current_byte_iter_count + self.lsb_c];

                            if let Some(pixel_to_modify) = pixel_iter.next() {
                                pixel_iter_counter = pixel_iter_counter - 1;
                                let mut color_change = ColorChange(
                                    pixel_to_modify.0,
                                    pixel_to_modify.1,
                                    pixel_to_modify.2.clone().into(),
                                    Rgb::from([0, 0, 0]),
                                );
                                let bits_to_modify = pixel_to_modify
                                    .2
                                    .channels_mut()
                                    .get_mut::<usize>(encoding_channel)
                                    .unwrap()
                                    .view_bits_mut::<Lsb0>();

                                put_bits(bits_to_encode_slice, bits_to_modify, &self.lsb_c);
    
                                color_change.3 = pixel_to_modify.2.clone().into();
                                current_byte_map.affected_points.push(color_change);
                                current_byte_iter_count += self.lsb_c;
                            } else {
                                break 'data_iter;
                            }
                        }
                    }

                    encode_maps.push(current_byte_map);
                }

                if self.spread {
                    if pixel_iter_counter == 0 {
                        break 'encode_rounds;
                    } else {
                        continue;
                    }
                } else {
                    if let Some(_padding_bits_value) = padding_bits {
                        // TODO: put trailing padding bytes
                        break 'encode_rounds;
                    } else {
                        break 'encode_rounds;
                    }
                }
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

// fn encode_bytes<'a>(bytes: &[u8], into_iter: impl Iterator<Item = (u32, u32, &'a mut Rgb<u8>)>) {}

fn put_bits(bits: &BitSlice<Lsb0, u8>, into: &mut BitSlice<Lsb0, u8>, lsb_c: &usize) {
    for i in 0..*lsb_c {
        into.set(i, bits[i]);
    }
}

impl ImageRules for ImageEncoder {
    /// Skip the first `offset` bytes in the source buffer
    fn set_offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Sets the number of least significative bits to edit for each
    /// byte in the source buffer. The default is 1. The higher the value gets
    /// the least space is required to encode data into the source, but the resulting
    /// image will get noticeably different from the original
    fn set_use_n_lsb(&mut self, n: usize) -> &mut Self {
        self.lsb_c = n;
        self
    }

    /// Specifies wich color channel will be the one used to store information bits.
    fn set_use_channel(&mut self, channel: RgbChannel) -> &mut Self {
        self.encoding_channel = channel;
        self
    }

    /// When encoding data, `n` pixels will be skipped after each edited pixel
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

    fn set_padding(&mut self, value: &str) -> &mut Self {
        self.padding = Some(String::from(value));
        self
    }
}

fn bytes_needed_for_data<R>(data: &[u8], rules: &R) -> usize
where
    R: ImageRules,
{
    (((data.len() * 8) - (rules.get_offset() * 3 * 8)) * rules.get_step_by_n_pixels())
        / rules.get_use_n_lsb()
    // total data bits   skipped pixels size in bits     iterator step size               bits used per pixel
}

#[allow(dead_code)]
fn eprint_color_changes(byte_map: &ByteEncodeMap, steps: usize) {
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
mod tests {
    fn ensure_out_dir() -> std::io::Result<()> {
        std::fs::create_dir_all("tests/out")
    }

    use crate::{encoder::ImageEncoder, prelude::*};

    #[test]
    fn target_byte_size_calc() {
        let mut encoder = ImageEncoder::default();
        assert_eq!(super::bytes_needed_for_data(&[8, 1, 2, 3], &encoder), 32);
        encoder.set_use_n_lsb(2);
        assert_eq!(super::bytes_needed_for_data(&[8, 1, 2, 3], &encoder), 16);
        encoder.set_step_by_n_pixels(2);
        assert_eq!(super::bytes_needed_for_data(&[8, 1, 2, 3], &encoder), 32);
    }

    #[test]
    fn simple_encoding() {
        ensure_out_dir().unwrap();

        let encode_result = super::ImageEncoder::from("tests/images/red_panda.jpg")
            .set_use_n_lsb(2)
            .set_use_channel(RgbChannel::Blue)
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
            .save("tests/out/red_panda_steg.jpeg", ImageFormat::Jpeg)
            .expect("Could not create output file");
    }
}
