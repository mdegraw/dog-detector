use image::{ImageBuffer, Rgb};

pub type Frame = Vec<u8>;
pub type FrameBuffer = ImageBuffer<Rgb<u8>, Frame>;

/// Converts RGB pixels to a SSD1306 OLED byte array
pub fn image_buffer_to_oled_byte_array(frame_buffer: &FrameBuffer, threshold: u8) -> Frame {
    let resized_img =
        image::imageops::resize(frame_buffer, 128, 64, image::imageops::FilterType::Nearest);

    let (_, _, _, _, byte_array) = resized_img.chunks(3).fold(
        (0, 0, 7_i32, resized_img.len(), Vec::<u8>::new()),
        |(mut number, mut i, mut byte_index, pixels_len, mut oled_frame), rgb| {
            // Get the average of the RGB
            let avg: u8 = rgb.iter().sum::<u8>() / 3;

            // TODO: Adjust based on lighting, day/night and IR camera
            // Black and white threshold default 128?
            if avg > threshold {
                number += 2_u8.pow(byte_index as u32);
            }

            byte_index -= 1;

            // if this was the last pixel of a row or the last pixel of the
            // image, fill up the rest of our byte with zeros so it always contains 8 bits
            if (i != 0 && (((i / 3) + 1) % (128)) == 0) || (i == (pixels_len - 3)) {
                byte_index = -1;
            }

            // // When there are 8 bits push into Vec
            if byte_index < 0 {
                // Push inverted color
                // oled_frame.push(255 - number);
                oled_frame.push(number);
                number = 0;
                byte_index = 7;
            }

            i += 3;

            (number, i, byte_index, pixels_len, oled_frame)
        },
    );

    byte_array
}
