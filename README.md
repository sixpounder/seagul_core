A library for encoding arbitrary data
into images, a tecnique also known as *steganography*.

This is the foundation for the `seagul` cli application.

# Basic example

## Encode

Read an image and resave it with some verses encoded into it, using the
last 2 bits on the blue channel of each pixel to encode them

```rust
let encode_result = super::ImageEncoder::from("source.png")
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
    .save("encoded.png", ImageFormat::Png)
    .expect("Could not create output file");
```

## Decode

```rust
let decoded = ImageDecoder::from("encoded.png")
    .set_use_n_lsb(2)
    .set_use_channel(RgbChannel::Blue)
    .until_marker(Some(b"way.")) // <- If you know how the message ends
    .decode();

assert!(decoded.is_ok());

let decoded = decoded.unwrap().as_raw();

println!("Raw decoded:\n{}", decoded_string);
```

# Supported formats
While almost every major image format is supported as input, at the moment only
PNG and BMP are supported as output formats. JPEG and other formats support is planned.
