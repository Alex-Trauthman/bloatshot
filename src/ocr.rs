use anyhow::{Result, anyhow};
use image::{GenericImageView, Luma};
use leptess::tesseract;
use std::path::{Path, PathBuf};

/// Processes an image to improve OCR accuracy and returns the path to the processed file.
pub fn preprocess_image(input_path: &Path, scale: f32) -> Result<PathBuf> {
    let img = image::open(input_path)
        .map_err(|e| anyhow!("Failed to open image for preprocessing: {}", e))?;

    let mut processed = img.grayscale();
    let (width, height) = processed.dimensions();
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    processed = processed.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    let mut luma_img = processed.to_luma8();
    for pixel in luma_img.pixels_mut() {
        if pixel[0] > 128 {
            *pixel = Luma([255]);
        } else {
            *pixel = Luma([0]);
        }
    }

    let output_path = input_path.with_extension("processed.png");
    luma_img
        .save(&output_path)
        .map_err(|e| anyhow!("Failed to save processed image: {}", e))?;

    Ok(output_path)
}

/// Runs Tesseract OCR on the provided image path using the specified language.
pub fn perform_ocr(img_path: &Path, lang: &str, scale: f32) -> Result<String> {
    let processed_path = preprocess_image(img_path, scale)?;
    let mut api = tesseract::TessApi::new(None, lang)
        .map_err(|e| anyhow!("Failed to initialize Tesseract: {}", e))?;

    let pix = leptess::leptonica::pix_read(&processed_path)
        .map_err(|e| anyhow!("Failed to read image for OCR: {}", e))?;

    api.set_image(&pix);
    let text = api
        .get_utf8_text()
        .map_err(|e| anyhow!("OCR error: {}", e))?;

    Ok(text.trim().to_string())
}
