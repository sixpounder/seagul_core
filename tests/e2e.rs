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

    let verses = b"Midway upon the journey of our life
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
In which I had abandoned the true way.--";

    let encode_result = JpegEncoder::from("tests/images/red_panda.jpg")
        .offset(0)
        .use_n_lsb(2)
        // .source_data(source_data)
        .encode_data(verses);

    if let Err(e) = encode_result {
        panic!("{}", e.as_str());
    }

    encode_result
        .unwrap()
        .save("tests/out/red_panda_steg.png")
        .expect("Could not create output file");

    let mut created_image =
        File::open("tests/out/red_panda_steg.png").expect("Failed to open created image");
    let mut source_data: Vec<u8> = Vec::new();
    created_image
        .read_to_end(&mut source_data)
        .expect("Cannot read file");

    let decoded = JpegDecoder::new()
        .offset(0)
        .use_n_lsb(2)
        .until_marker(b"--")
        .decode_buffer(source_data.as_bytes());

    assert!(decoded.is_ok());

    let decoded = decoded.unwrap();
    let decoded_string = decoded.as_raw();
    let original = String::from_utf8_lossy(verses);

    println!("Raw decoded:\n{}", decoded_string);

    assert_eq!(decoded.hit_marker(), true);
    assert_eq!(decoded_string.len(), verses.len());
    assert_eq!(decoded_string.cmp(&original), Ordering::Equal);
}
