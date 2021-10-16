use core::panic;
use std::fs::File;

use seagul_core::{decoder::ImageDecoder, prelude::*};
use seagul_core::encoder::ImageEncoder;

fn ensure_out_dir() -> std::io::Result<()> {
    std::fs::create_dir_all("tests/out")
}

#[test]
fn encode_bytes() {
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

    let encode_result = ImageEncoder::from("tests/images/red_panda.jpg")
        .set_offset(0)
        .set_use_n_lsb(2)
        .encode_bytes(verses);

    if let Err(e) = encode_result {
        panic!("{}", e.as_str());
    }

    encode_result
        .unwrap()
        .save("tests/out/red_panda_steg.png", ImageFormat::Png)
        .expect("Could not create output file");

    let mut created_image =
        File::open("tests/out/red_panda_steg.png").expect("Failed to open created image");

    let decoded = ImageDecoder::from(&mut created_image)
        .set_offset(0)
        .set_use_n_lsb(2)
        .until_marker(Some(b"--"))
        .decode();

    assert!(decoded.is_ok());

    let decoded = decoded.unwrap();
    let decoded_string = decoded.as_raw();

    println!("Raw decoded:\n{}", decoded_string);

    assert_eq!(decoded.hit_marker(), true);
}

#[test]
fn encode_bytes_spread() {
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

    let encode_result = ImageEncoder::from("tests/images/red_panda.jpg")
        .set_offset(0)
        .set_spread(true)
        .set_use_n_lsb(2)
        .encode_bytes(verses);

    if let Err(e) = encode_result {
        panic!("{}", e.as_str());
    }

    encode_result
        .unwrap()
        .save("tests/out/red_panda_spread.png", ImageFormat::Png)
        .expect("Could not create output file");

    let mut created_image =
        File::open("tests/out/red_panda_spread.png").expect("Failed to open created image");

    let decoded = ImageDecoder::from(&mut created_image)
        .set_offset(0)
        .set_use_n_lsb(2)
        .decode();

    assert!(decoded.is_ok());

    let decoded = decoded.unwrap();
    let decoded_string = decoded.as_raw();

    println!("Raw decoded:\n{}", decoded_string);

    assert_eq!(decoded.hit_marker(), false);
}