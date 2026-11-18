use image::ImageReader;
use std::io::Cursor;

pub fn compress_image(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let image = ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    // Resize if necessary, preserving aspect ratio.
    let image = image.resize(2560, 2560, image::imageops::FilterType::Lanczos3);

    // Encode as WebP.
    let encoder = webp::Encoder::from_image(&image)?;

    let compressed = encoder.encode(85.0);

    Ok(compressed.to_vec())
}
