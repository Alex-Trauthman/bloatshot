use image::{DynamicImage, GenericImageView, Pixel};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum UIElement {
    Button,
    Input,
    Checkbox,
    Dropdown,
    Text,
}

impl UIElement {
    pub fn as_label(&self) -> &'static str {
        match self {
            UIElement::Button => "Button",
            UIElement::Input => "Input",
            UIElement::Checkbox => "Checkbox",
            UIElement::Dropdown => "Dropdown",
            UIElement::Text => "Text",
        }
    }
}

pub struct SemanticEngine;

impl SemanticEngine {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn classify_region(&self, img: &DynamicImage, x: u32, y: u32, w: u32, h: u32) -> UIElement {
        let (img_w, img_h) = img.dimensions();
        
        // Increase padding to catch web borders which are often further from text
        let padding = 8;
        let x1 = x.saturating_sub(padding);
        let y1 = y.saturating_sub(padding);
        let x2 = (x + w + padding).min(img_w - 1);
        let y2 = (y + h + padding).min(img_h - 1);

        if x2 <= x1 || y2 <= y1 {
            return UIElement::Text;
        }

        // 1. Check for Checkbox/Radio (Small square/circle near text)
        if w < 40 && h < 40 && (w as f32 / h as f32).abs() - 1.0 < 0.3 {
            return UIElement::Checkbox;
        }

        // 2. Sample background color at center (where text is)
        let center_pixel = img.get_pixel(x + w/2, y + h/2).to_luma().0[0];
        
        // 3. Sample border colors
        let mut border_lumas = Vec::new();
        for px in x1..=x2 {
            border_lumas.push(img.get_pixel(px, y1).to_luma().0[0]);
            border_lumas.push(img.get_pixel(px, y2).to_luma().0[0]);
        }
        for py in y1..=y2 {
            border_lumas.push(img.get_pixel(x1, py).to_luma().0[0]);
            border_lumas.push(img.get_pixel(x2, py).to_luma().0[0]);
        }

        let sum: u32 = border_lumas.iter().map(|&l| l as u32).sum();
        let avg_border = sum / border_lumas.len() as u32;
        let var_sum: u32 = border_lumas.iter().map(|&l| (l as i32 - avg_border as i32).pow(2) as u32).sum();
        let variance = var_sum / border_lumas.len() as u32;

        // 4. Semantic Heuristics for Web Components
        
        // High Contrast between text background and border area usually means a container
        let contrast = (center_pixel as i32 - avg_border as i32).abs();

        if variance < 300 { // Border area is relatively solid
            // Dropdown detection (check for the 'V' or arrow spike on the right)
            let right_check = x2.saturating_sub(4);
            let mut vertical_lines = 0;
            for py in y1..y2 {
                let p = img.get_pixel(right_check, py).to_luma().0[0];
                if (p as i32 - avg_border as i32).abs() > 40 {
                    vertical_lines += 1;
                }
            }
            if vertical_lines > (y2 - y1) / 2 {
                return UIElement::Dropdown;
            }

            if contrast > 15 {
                // Background of element is different from surrounding
                if center_pixel > 230 {
                    return UIElement::Input; // Typically white background fields
                } else {
                    return UIElement::Button; // Colored or gray buttons
                }
            } else if avg_border < 200 {
                 // Even if contrast is low, a dark solid box on a light page is likely a button/pill
                 return UIElement::Button;
            }
        }

        UIElement::Text
    }
}
