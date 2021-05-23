use core::panic;
use std::{cmp::Ordering, fs::File, io::Read};

use image::EncodableLayout;
use seagul::{decoder::JpegDecoder, prelude::*};
use seagul::encoder::JpegEncoder;

fn ensure_out_dir() -> std::io::Result<()> {
    std::fs::create_dir_all("tests/out")
}

#[test]
fn encode_sample_image() {
    ensure_out_dir().expect("Could not create output directory");

    // let verses = b"
    //     Midway upon the journey of our life
    //     I found myself within a forest dark,
    //     For the straightforward pathway had been lost.
    //     Ah me! how hard a thing it is to say
    //     What was this forest savage, rough, and stern,
    //     Which in the very thought renews the fear.
    //     So bitter is it, death is little more;
    //     But of the good to treat, which there I found,
    //     Speak will I of the other things I saw there.
    //     I cannot well repeat how there I entered,
    //     So full was I of slumber at the moment
    //     In which I had abandoned the true way.
    // ";

    let verses = b"abcd";

    // let mut file = File::open("tests/images/small.jpeg").expect("Test image not found");
    // let mut source_data: Vec<u8> = Vec::new();
    // file.read_to_end(&mut source_data)
    //     .expect("Cannot test image");

    // image::load_from_memory(source_data.as_bytes()).unwrap().save("tests/out/unmodified.jpeg").unwrap();

    let encode_result = JpegEncoder::from("tests/images/small.jpeg")
        .offset(0)
        .use_n_lsb(2)
        // .source_data(source_data)
        .encode_data(verses);

    if let Err(e) = encode_result {
        panic!("{}", e.as_str());
    }

    encode_result
        .unwrap()
        .save("tests/out/small_steg.jpeg")
        .expect("Could not create output file");

    let mut created_image =
        File::open("tests/out/small_steg.jpeg").expect("Failed to open created image");
    let mut source_data: Vec<u8> = Vec::new();
    created_image
        .read_to_end(&mut source_data)
        .expect("Cannot read file");

    let decoded = JpegDecoder::new()
        .offset(0)
        .use_n_lsb(2)
        .until_marker(b"cd")
        .decode_buffer(source_data.as_bytes());

    assert!(decoded.is_ok());

    let decoded = decoded.unwrap();
    println!("Raw decoded: {}", decoded.as_raw());
    assert_eq!(decoded.hit_marker(), true);

    let expected_str = decoded.as_string();
    let original = String::from_utf8_lossy(verses);
    println!("{}", &expected_str);
    assert_eq!(expected_str.len(), verses.len());
    assert_eq!(expected_str.cmp(&original), Ordering::Equal);
}
