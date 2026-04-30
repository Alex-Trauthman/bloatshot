use anyhow::{Result, anyhow};
use image::{DynamicImage, GenericImageView};
use ocr_rs::{OcrEngine, OcrResult_ as OcrResult};
use std::collections::HashMap;
use std::path::PathBuf;

fn get_model_path(name: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").map_err(|_| anyhow!("HOME env var not set"))?;
    Ok(PathBuf::from(home)
        .join(".local/share/bloatshot")
        .join(name))
}

/// Standard OCR using PP-OCRv5 (multilingual: Chinese, English, Japanese, etc.)
pub fn perform_standard_ocr_raw(img: &image::DynamicImage) -> Result<Vec<OcrResult>> {
    let det_path = get_model_path("det.mnn")?;
    let rec_path = get_model_path("rec.mnn")?;
    let keys_path = get_model_path("ppocr_keys.txt")?;

    let engine = OcrEngine::new(
        det_path.to_str().unwrap(),
        rec_path.to_str().unwrap(),
        keys_path.to_str().unwrap(),
        None,
    )
    .map_err(|e| anyhow!("PaddleOCR engine init failed: {:?}", e))?;

    engine
        .recognize(img)
        .map_err(|e| anyhow!("PaddleOCR recognition failed: {:?}", e))
}

pub fn perform_standard_ocr(img: &DynamicImage) -> Result<String> {
    let results = perform_standard_ocr_raw(img)?;
    if results.is_empty() {
        return Ok("".to_string());
    }

    // Group results into rows by their Y-coordinate to preserve layout
    let mut rows: Vec<Vec<&OcrResult>> = Vec::new();
    let mut sorted_results: Vec<&OcrResult> = results.iter().collect();
    // Sort by Y first, then X.
    sorted_results.sort_by(|a, b| {
        let ay = a.bbox.rect.top();
        let by = b.bbox.rect.top();
        if (ay - by).abs() < 8 {
            a.bbox.rect.left().partial_cmp(&b.bbox.rect.left()).unwrap()
        } else {
            ay.partial_cmp(&by).unwrap()
        }
    });

    for result in sorted_results {
        if let Some(last_row) = rows.last_mut() {
            let row_y = last_row[0].bbox.rect.top();
            if (result.bbox.rect.top() - row_y).abs() < 12 {
                last_row.push(result);
                continue;
            }
        }
        rows.push(vec![result]);
    }

    let mut output = String::new();
    for row in rows {
        let mut row_text = String::new();
        let mut last_right = -1;
        
        for (j, res) in row.iter().enumerate() {
            let left = res.bbox.rect.left();
            if j > 0 {
                let gap = left - last_right;
                if gap > 25 {
                    row_text.push('\t');
                } else {
                    row_text.push(' ');
                }
            }
            row_text.push_str(res.text.trim());
            last_right = res.bbox.rect.right();
        }
        output.push_str(&row_text);
        output.push('\n');
    }

    Ok(output.trim().to_string())
}

/// Math/LaTeX OCR using RapidLaTeXOCR (ViT encoder + transformer decoder)
pub fn perform_semantic_ocr(img: &DynamicImage) -> Result<String> {
    use ort::session::Session;
    use ort::value::Tensor;

    // Load tokenizer vocabulary (id -> token mapping)
    let tokenizer_path = get_model_path("math_tokenizer.json")?;
    let tokenizer_json = std::fs::read_to_string(&tokenizer_path)
        .map_err(|e| anyhow!("Failed to read tokenizer: {}", e))?;
    let tokenizer: serde_json::Value = serde_json::from_str(&tokenizer_json)
        .map_err(|e| anyhow!("Failed to parse tokenizer JSON: {}", e))?;

    // Build id-to-token map from vocab
    let vocab = tokenizer["model"]["vocab"]
        .as_object()
        .ok_or_else(|| anyhow!("Invalid tokenizer: missing model.vocab"))?;
    let mut id_to_token: HashMap<i64, String> = HashMap::new();
    for (token, id) in vocab {
        if let Some(id_num) = id.as_i64() {
            id_to_token.insert(id_num, token.clone());
        }
    }

    let bos_id: i64 = 1; // [BOS]
    let eos_id: i64 = 2; // [EOS]
    let max_len: usize = 512;
    let decoder_vocab_size: usize = 8000; // from model output shape

    // Load ONNX models
    let mut encoder = Session::builder()?.commit_from_file(get_model_path("math_encoder.onnx")?)?;
    let mut decoder = Session::builder()?.commit_from_file(get_model_path("math_decoder.onnx")?)?;

    // Crop 2 pixels from edges to remove potential selection/window borders
    let (orig_w, orig_h) = img.dimensions();
    let img = if orig_w > 10 && orig_h > 10 {
        DynamicImage::ImageRgba8(img.view(2, 2, orig_w - 4, orig_h - 4).to_image())
    } else {
        img.clone()
    };

    // Preprocess: convert to grayscale
    let gray = img.to_luma8();
    let (w, h) = gray.dimensions();

    // Resize to height 64 while maintaining aspect ratio
    let target_h: u32 = 64;
    let target_w: u32 = ((w as f32 / h as f32) * target_h as f32).round() as u32;
    // Round to nearest multiple of 32 (encoder patch size requirement)
    let target_w = ((target_w + 31) / 32 * 32).clamp(32, 512);

    let resized = image::imageops::resize(
        &gray,
        target_w,
        target_h,
        image::imageops::FilterType::Lanczos3,
    );

    // Normalize to [-1, 1] and pack into NCHW tensor
    let mut pixel_data: Vec<f32> = Vec::with_capacity((target_h * target_w) as usize);
    for y in 0..target_h {
        for x in 0..target_w {
            let val = resized.get_pixel(x, y)[0] as f32 / 255.0;
            pixel_data.push(val * 2.0 - 1.0); // normalize to [-1, 1]
        }
    }

    // Encoder input: "input" with shape [1, 1, H, W] (grayscale)
    let input_tensor = Tensor::from_array((
        [1usize, 1, target_h as usize, target_w as usize],
        pixel_data,
    ))?;
    let encoder_outputs = encoder.run(ort::inputs!["input" => input_tensor])?;

    // Encoder output: shape [1, seq_len, 256]
    let context_tensor = &encoder_outputs[0];
    let context_extracted = context_tensor.try_extract_tensor::<f32>()?;
    let context_shape = &context_extracted.0;
    let context_data = context_extracted.1;
    let context_seq_len = context_shape[1] as usize;
    let context_hidden = context_shape[2] as usize;

    // Autoregressive decoding
    let mut generated_ids: Vec<i64> = vec![bos_id];

    for _ in 0..max_len {
        let seq_len = generated_ids.len();

        // Decoder input "x": token ids [1, seq_len] (INT64)
        let x_tensor = Tensor::from_array(([1usize, seq_len], generated_ids.clone()))?;

        // Decoder input "mask": attention mask [1, seq_len] (BOOL - all true)
        let mask_data: Vec<bool> = vec![true; seq_len];
        let mask_tensor = Tensor::from_array(([1usize, seq_len], mask_data))?;

        // Decoder input "context": encoder hidden states (FLOAT)
        let context_copy = Tensor::from_array((
            [1usize, context_seq_len, context_hidden],
            context_data.to_vec(),
        ))?;

        let decoder_outputs = decoder.run(ort::inputs![
            "x" => x_tensor,
            "mask" => mask_tensor,
            "context" => context_copy
        ])?;

        // Output shape: [1, seq_len, 8000]
        let logits_extracted = decoder_outputs[0].try_extract_tensor::<f32>()?;
        let logits_data = logits_extracted.1;

        // Get logits for the last token position
        let offset = (seq_len - 1) * decoder_vocab_size;

        // Greedy argmax decoding
        let mut best_id: i64 = 0;
        let mut best_val = f32::NEG_INFINITY;
        for v in 0..decoder_vocab_size {
            let val = logits_data[offset + v];
            if val > best_val {
                best_val = val;
                best_id = v as i64;
            }
        }

        if best_id == eos_id {
            break;
        }
        generated_ids.push(best_id);
    }

    // Decode tokens to LaTeX string (skip BOS)
    let mut latex = String::new();
    for &id in &generated_ids[1..] {
        if let Some(token) = id_to_token.get(&id) {
            let clean = token.replace('\u{0120}', ""); // Remove BPE space markers
            latex.push_str(&clean);
        }
    }

    // Post-process: carefully remove spaces while preserving them after backslash commands if no brace follows
    let mut cleaned = String::new();
    let chars: Vec<char> = latex.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ' ' {
            // Only keep space if it follows a command word and is not followed by a brace
            let mut is_cmd = false;
            if i > 0 {
                let mut j = i - 1;
                while j > 0 && chars[j].is_alphabetic() { j -= 1; }
                if chars[j] == '\\' { is_cmd = true; }
            }
            
            let next_is_brace = i + 1 < chars.len() && (chars[i+1] == '{' || chars[i+1] == '(');
            if is_cmd && !next_is_brace {
                cleaned.push(' ');
            }
        } else {
            cleaned.push(chars[i]);
        }
        i += 1;
    }

    // Function to strip outer braces from a string if they are balanced
    fn strip_balanced(s: &str) -> String {
        if !s.starts_with('{') || !s.ends_with('}') { return s.to_string(); }
        let mut count = 0;
        for (i, c) in s.chars().enumerate() {
            if c == '{' { count += 1; }
            else if c == '}' {
                count -= 1;
                if count == 0 && i < s.len() - 1 { return s.to_string(); }
            }
        }
        if count == 0 {
            s[1..s.len()-1].to_string()
        } else {
            s.to_string()
        }
    }

    // Strip only the outermost braces of the entire formula if balanced
    let mut final_out = strip_balanced(&cleaned);

    // Final "Paper Perfection" Step: Upgrade ( ... ) to \left( ... \right) if it contains a \frac
    if final_out.contains("\\frac") && final_out.contains("(") && final_out.contains(")") {
        final_out = final_out.replace("(", "\\left(").replace(")", "\\right)");
    }

    Ok(final_out.trim().to_string())
}

/// Table OCR — uses standard PaddleOCR with a heuristic to reconstruct the table structure.
pub fn perform_table_ocr(img: &DynamicImage) -> Result<String> {
    let results = perform_standard_ocr_raw(img)?;
    if results.is_empty() {
        return Ok("".to_string());
    }

    // Heuristic: Group results into rows by their Y-coordinate.
    // We use a small threshold to allow for slight misalignments.
    let mut rows: Vec<Vec<&OcrResult>> = Vec::new();
    let mut sorted_results: Vec<&OcrResult> = results.iter().collect();
    // Sort by Y first, then X.
    sorted_results.sort_by(|a, b| {
        let ay = a.bbox.rect.top();
        let by = b.bbox.rect.top();
        if (ay - by).abs() < 5 {
            a.bbox.rect.left().partial_cmp(&b.bbox.rect.left()).unwrap()
        } else {
            ay.partial_cmp(&by).unwrap()
        }
    });

    for result in sorted_results {
        if let Some(last_row) = rows.last_mut() {
            let row_y = last_row[0].bbox.rect.top();
            if (result.bbox.rect.top() - row_y).abs() < 10 {
                last_row.push(result);
                continue;
            }
        }
        rows.push(vec![result]);
    }

    // Find max columns
    let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);

    // If it's just a single column, don't use table formatting
    if max_cols <= 1 {
        let mut text = String::new();
        for row in rows {
            for res in row {
                text.push_str(&res.text);
                text.push(' ');
            }
            text.push('\n');
        }
        return Ok(text.trim().to_string());
    }

    // Format as Markdown table
    let mut markdown = String::new();
    for (i, row) in rows.iter().enumerate() {
        let mut row_text: Vec<String> = row.iter().map(|r| r.text.trim().to_string()).collect();
        // Pad with empty strings to match max_cols
        while row_text.len() < max_cols {
            row_text.push("".to_string());
        }

        markdown.push_str("| ");
        markdown.push_str(&row_text.join(" | "));
        markdown.push_str(" |\n");

        if i == 0 {
            // Add separator
            markdown.push('|');
            for _ in 0..max_cols {
                markdown.push_str(" --- |");
            }
            markdown.push('\n');
        }
    }

    Ok(markdown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_extract_ocr_on_stock_image() {
        // Tests `bloatshot -e` functionality
        crate::util::ensure_onnx_models().expect("Failed to download models");

        let home = std::env::var("HOME").expect("HOME env var not set");
        let image_path = PathBuf::from(&home).join("bloatshots/2026-04-28/22-46-44.png");
        let target_path = PathBuf::from(&home).join("bloatshot/target.txt");

        if !image_path.exists() || !target_path.exists() {
            println!("Test skipped: stock image or target.txt not found");
            return;
        }

        let expected = fs::read_to_string(&target_path).expect("Failed to read target.txt");
        let img = image::open(&image_path).expect("Failed to open test image");

        let result = perform_standard_ocr(&img).expect("perform_standard_ocr failed");

        println!("--- EXPECTED (from target.txt) ---");
        println!("{}", expected);
        println!("\n--- ACTUAL (-e standard OCR) ---");
        println!("{}", result);

        assert!(!result.is_empty(), "OCR produced empty string!");
        assert!(
            result.to_lowercase().contains("bloatshot"),
            "OCR output missing 'bloatshot'. Got:\n{}",
            result
        );
    }

    #[test]
    fn test_math_ocr_on_formula_image() {
        // Tests `bloatshot -m` functionality with a real LaTeX formula
        crate::util::ensure_onnx_models().expect("Failed to download models");

        let home = std::env::var("HOME").expect("HOME env var not set");
        let image_path = PathBuf::from(&home).join("bloatshots/2026-04-28/23-27-49.png");

        if !image_path.exists() {
            println!("Test skipped: math formula image not found");
            return;
        }

        let img = image::open(&image_path).expect("Failed to open test image");
        let result = perform_semantic_ocr(&img).expect("perform_semantic_ocr failed");

        println!("\n--- ACTUAL (-m math OCR) ---");
        println!("{}", result);

        assert!(!result.is_empty(), "Math OCR produced empty string!");
        assert!(
            result.contains("\\int") || result.contains("\\sum") || result.contains("\\frac"),
            "Math OCR output missing LaTeX commands. Got:\n{}",
            result
        );
    }

    #[test]
    fn test_attention_formula() {
        crate::util::ensure_onnx_models().expect("Failed to download models");
        let image_path = PathBuf::from("/home/alekstrautima/bloatshots/2026-04-29/22-32-51.png");

        if !image_path.exists() {
            println!("Test skipped: attention image not found");
            return;
        }

        let img = image::open(&image_path).expect("Failed to open test image");
        let result = perform_semantic_ocr(&img).expect("perform_semantic_ocr failed");

        println!("\n--- ATTENTION OCR RESULT ---");
        println!("{}", result);

        assert!(result.contains("Attention"), "Result missing 'Attention'");
        assert!(result.contains("softmax"), "Result missing 'softmax'");
        assert!(result.contains("QK"), "Result missing 'QK'");
    }

    #[test]
    fn test_semantic_ocr_fallback_on_text() {
        // Tests that `-m` also works on regular text images (falls back gracefully)
        crate::util::ensure_onnx_models().expect("Failed to download models");

        let home = std::env::var("HOME").expect("HOME env var not set");
        let image_path = PathBuf::from(&home).join("bloatshots/2026-04-28/22-46-44.png");

        if !image_path.exists() {
            println!("Test skipped: stock image not found");
            return;
        }

        let img = image::open(&image_path).expect("Failed to open test image");
        // Math OCR on a non-math image should still produce something (even if garbled)
        let result = perform_semantic_ocr(&img);
        assert!(
            result.is_ok(),
            "Semantic OCR crashed on text image: {:?}",
            result.err()
        );

        println!("\n--- ACTUAL (-m on text image, should be LaTeX-ish) ---");
        println!("{}", result.unwrap());
    }

    #[test]
    fn test_table_ocr_on_sample_image() {
        // Tests `bloatshot -t` functionality
        crate::util::ensure_onnx_models().expect("Failed to download models");

        let home = std::env::var("HOME").expect("HOME env var not set");
        let image_path = PathBuf::from(&home).join("bloatshots/2026-04-28/table_test.jpg");

        if !image_path.exists() {
            println!("Test skipped: table test image not found");
            return;
        }

        let img = image::open(&image_path).expect("Failed to open test image");
        let result = perform_table_ocr(&img).expect("perform_table_ocr failed");

        println!("\n--- ACTUAL (-t table OCR) ---");
        println!("{}", result);

        assert!(!result.is_empty(), "Table OCR produced empty string!");
        assert!(
            result.contains("|"),
            "Table OCR output missing Markdown pipe characters!"
        );
        assert!(
            result.to_lowercase().contains("alice"),
            "Table OCR output missing 'Alice'!"
        );
    }
}
