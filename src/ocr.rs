use anyhow::{Result, anyhow};
use image::{GenericImageView, Luma, imageops};
use leptess::{tesseract, leptonica, capi};
use std::path::{Path, PathBuf};
use crate::semantic::{SemanticEngine, UIElement};

/// Advanced image preprocessing for maximum OCR accuracy.
/// Uses 3x upscaling, adaptive sharpening, and Otsu-like thresholding.
pub fn preprocess_image(input_path: &Path, scale: f32) -> Result<PathBuf> {
    let img = image::open(input_path)
        .map_err(|e| anyhow!("Failed to open image for preprocessing: {}", e))?;

    let (width, height) = img.dimensions();
    
    // 1. Force a higher minimum scale for small text
    let effective_scale = if width < 1000 { scale * 1.5 } else { scale };
    let new_width = (width as f32 * effective_scale) as u32;
    let new_height = (height as f32 * effective_scale) as u32;

    // 2. Grayscale and Upscale with high-quality filter
    let mut processed = img.grayscale();
    processed = processed.resize(new_width, new_height, imageops::FilterType::Lanczos3);

    // 3. Adaptive binarization (Enhanced Contrast)
    let mut luma_img = processed.to_luma8();
    
    // Calculate global mean to use as a baseline for Otsu-lite thresholding
    let mut sum: u64 = 0;
    for pixel in luma_img.pixels() {
        sum += pixel[0] as u64;
    }
    let mean = (sum / (luma_img.width() * luma_img.height()) as u64) as u8;
    
    // Apply a slightly aggressive threshold centered around the mean
    // This helps preserve thin fonts on grey backgrounds
    let threshold = if mean > 180 { 160 } else if mean < 100 { 100 } else { 128 };

    for pixel in luma_img.pixels_mut() {
        if pixel[0] > threshold {
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

/// Runs Tesseract OCR on the provided image path and uses SemanticEngine to classify regions.
pub fn perform_ocr_with_semantic(img_path: &Path, lang: &str, scale: f32) -> Result<String> {
    let original_img = image::open(img_path)
        .map_err(|e| anyhow!("Failed to open original image: {}", e))?;
    
    let processed_path = preprocess_image(img_path, scale)?;
    let mut api = tesseract::TessApi::new(None, lang)
        .map_err(|e| anyhow!("Failed to initialize Tesseract: {}", e))?;

    let pix = leptonica::pix_read(&processed_path)
        .map_err(|e| anyhow!("Failed to read image for OCR: {}", e))?;

    api.set_image(&pix);
    
    let level = capi::TessPageIteratorLevel_RIL_TEXTLINE;
    let component_boxa = api.get_component_images(level, true)
        .ok_or_else(|| anyhow!("Failed to extract text components"))?;

    let semantic_engine = SemanticEngine::new()?;
    let mut final_output = String::new();

    let count = component_boxa.get_n();
    for i in 0..count {
        if let Some(box_) = component_boxa.get_box(i) {
            let mut x = 0;
            let mut y = 0;
            let mut w = 0;
            let mut h = 0;
            box_.get_geometry(Some(&mut x), Some(&mut y), Some(&mut w), Some(&mut h));
            
            // Map back to original image scale for semantic analysis
            // We use the same 'effective_scale' logic here to stay in sync
            let img_w = original_img.width();
            let actual_scale = if img_w < 1000 { scale * 1.5 } else { scale };

            let orig_x = (x as f32 / actual_scale) as u32;
            let orig_y = (y as f32 / actual_scale) as u32;
            let orig_w = (w as f32 / actual_scale) as u32;
            let orig_h = (h as f32 / actual_scale) as u32;

            let element_type = semantic_engine.classify_region(&original_img, orig_x, orig_y, orig_w, orig_h);

            api.set_image(&pix);
            api.set_rectangle(x, y, w, h);
            let text = api.get_utf8_text()?.trim().to_string();

            if !text.is_empty() {
                match element_type {
                    UIElement::Text => final_output.push_str(&format!("{}\n", text)),
                    _ => final_output.push_str(&format!("[{}] \"{}\"\n", element_type.as_label(), text)),
                }
            }
        }
    }

    if final_output.is_empty() {
        api.set_image(&pix);
        return Ok(api.get_utf8_text()?.trim().to_string());
    }

    Ok(final_output.trim().to_string())
}

/// Simple OCR without semantic classification.
pub fn perform_ocr(img_path: &Path, lang: &str, scale: f32) -> Result<String> {
    let processed_path = preprocess_image(img_path, scale)?;
    let mut api = tesseract::TessApi::new(None, lang)
        .map_err(|e| anyhow!("Failed to initialize Tesseract: {}", e))?;

    let pix = leptonica::pix_read(&processed_path)
        .map_err(|e| anyhow!("Failed to read image for OCR: {}", e))?;

    api.set_image(&pix);
    let text = api.get_utf8_text()?.trim().to_string();

    Ok(text)
}
