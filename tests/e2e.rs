use std::{cmp::Ordering, fs::File, io::Read};

use image::EncodableLayout;
use seagul::{decoder::JpegDecoder, prelude::*};
use seagul::encoder::JpegEncoder;

#[test]
fn encode_sample_image() {
    let verses = "
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
        In which I had abandoned the true way.
    ";

    let mut file = File::open("tests/images/red_panda.jpg").expect("Test image not found");
    let mut source_data: Vec<u8> = Vec::new();
    file.read_to_end(&mut source_data)
        .expect("Cannot test image");

    let encode_result = JpegEncoder::new()
        .use_n_lsb(2)
        .source_data(source_data)
        .encode_data(verses.as_bytes());

    assert!(encode_result.is_ok(), "Encoding failed");

    encode_result
        .unwrap()
        .save("tests/out/steg.jpeg")
        .expect("Could not create output file");

    let mut created_image =
        File::open("tests/out/steg.jpeg").expect("Failed to open created image");
    let mut source_data: Vec<u8> = Vec::new();
    created_image
        .read_to_end(&mut source_data)
        .expect("Cannot read file");

    let decoded = JpegDecoder::new()
        .offset(0)
        .use_n_lsb(2)
        .until_marker(b"true way")
        .decode_buffer(source_data.as_bytes());

    assert!(decoded.is_ok());
    assert_eq!(decoded.as_ref().unwrap().hit_marker(), true);

    let decode_output = String::from_utf8(decoded.unwrap().data().clone());
    assert!(decode_output.is_ok());

    let expected_str = decode_output.unwrap();
    let original = String::from(verses);
    // println!("{}", original);
    // println!("{}", expected_str);
    assert_eq!(expected_str.len(), verses.len());
    assert_eq!(expected_str.cmp(&original), Ordering::Equal);
}
