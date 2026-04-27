use crate::ocr::perform_ocr_with_semantic;
use crate::util::{copy_to_clipboard, open_in_editor, send_notification};
use anyhow::{anyhow, Result};
use eframe::egui;
use image::GenericImageView;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};


#[derive(PartialEq, Clone, Copy)]
pub enum AppMode {
    Menu,
    Viewer,
}

#[derive(Clone, Copy)]
pub enum PendingAction {
    ExtractFull,
    SemanticFull,
    EditFull,
    SeeImage,
    SaveFull,
}

pub struct BloatshotApp {
    pub image_path: Option<PathBuf>,
    pub lang: String,
    pub scale: f32,
    preview_texture: Option<egui::TextureHandle>,
    ocr_result: Option<String>,
    selection_start: Option<egui::Pos2>,
    selection_end: Option<egui::Pos2>,
    pub is_selecting: bool,
    pub semantic_mode: bool,
    pub mode: AppMode,

    pub pending_action: Arc<Mutex<Option<PendingAction>>>,
}

impl BloatshotApp {
    pub fn new(
        lang: String,
        scale: f32,
        pending_action: Arc<Mutex<Option<PendingAction>>>,
        initial_mode: AppMode,
        initial_image: Option<PathBuf>,
    ) -> Self {
        Self {
            image_path: initial_image,
            lang,
            scale,
            preview_texture: None,
            ocr_result: None,
            selection_start: None,
            selection_end: None,
            is_selecting: false,
            semantic_mode: false,
            mode: initial_mode,
            pending_action,
        }
    }

    fn run_ocr_on_selection(&mut self, rect: egui::Rect, image_size: egui::Vec2) -> Result<()> {
        let path = self.image_path.as_ref().ok_or(anyhow!("No image"))?;
        let img = image::open(path)?;
        let (img_w, img_h) = img.dimensions();

        let x = (rect.min.x / image_size.x * img_w as f32) as u32;
        let y = (rect.min.y / image_size.y * img_h as f32) as u32;
        let w = (rect.width() / image_size.x * img_w as f32) as u32;
        let h = (rect.height() / image_size.y * img_h as f32) as u32;

        if w == 0 || h == 0 {
            return Ok(());
        }

        let sub_img = img.view(x, y, w, h).to_image();
        let sub_path = std::env::temp_dir().join("bloatshot_sub.png");
        sub_img.save(&sub_path)?;

        let text = if self.semantic_mode {
            perform_ocr_with_semantic(&sub_path, &self.lang, self.scale)?
        } else {
            crate::ocr::perform_ocr(&sub_path, &self.lang, self.scale)?
        };
        copy_to_clipboard(&text)?;
        self.ocr_result = Some(text);

        send_notification("OCR Complete", "Text copied to clipboard", Some(&sub_path));
        Ok(())
    }
}

impl eframe::App for BloatshotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if self.mode == AppMode::Menu {
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.heading("Bloatshot");
                ui.add_space(12.0);

                if ui.button("📋 Extract Text").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::ExtractFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("🧠 Semantic Extract").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::SemanticFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("💾 Save Image").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::SaveFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("🎨 Edit Image").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::EditFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("👁 See Image").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::SeeImage);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(12.0);
                if ui.button("❌ Cancel").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        } else {
            if self.preview_texture.is_none() {
                if let Some(path) = &self.image_path {
                    if let Ok(image_data) = image::open(path) {
                        let size = [image_data.width() as usize, image_data.height() as usize];
                        let pixels = image_data.to_rgba8();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            pixels.as_flat_samples().as_slice(),
                        );
                        self.preview_texture = Some(ui.ctx().load_texture(
                            "preview",
                            color_image,
                            Default::default(),
                        ));
                    }
                }
            }

            ui.horizontal(|ui| {
                if ui.button("⬅ Back").clicked() {
                    self.mode = AppMode::Menu;
                    self.preview_texture = None;
                    self.ocr_result = None;
                }
                if ui.button("Edit Full").clicked() {
                    if let Some(path) = &self.image_path {
                        open_in_editor(path).ok();
                    }
                }
                ui.checkbox(&mut self.semantic_mode, "🧠 Semantic");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

            if let Some(text) = &self.ocr_result {
                ui.add_space(5.0);
                ui.label("OCR Result (Copied):");
                let mut t = text.clone();
                if ui.text_edit_multiline(&mut t).changed() {
                    self.ocr_result = Some(t.clone());
                    let _ = copy_to_clipboard(&t);
                }
            }

            ui.add_space(5.0);
            ui.separator();
            if let Some(texture) = &self.preview_texture {
                let available_size = ui.available_size();
                let texture_size = texture.size_vec2();
                let ratio = (available_size.x / texture_size.x)
                    .min(available_size.y / texture_size.y)
                    .min(1.0);
                let display_size = texture_size * ratio;

                let (rect, response) = ui.allocate_exact_size(display_size, egui::Sense::drag());
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                if response.drag_started() {
                    self.selection_start =
                        Some(response.interact_pointer_pos().unwrap() - rect.min.to_vec2());
                    self.is_selecting = true;
                }
                if self.is_selecting {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.selection_end = Some(pos - rect.min.to_vec2());
                    }
                }
                if response.drag_stopped() {
                    self.is_selecting = false;
                    if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                        let selection_rect = egui::Rect::from_two_pos(start, end);
                        self.run_ocr_on_selection(selection_rect, display_size).ok();
                    }
                }
                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                    let draw_rect = egui::Rect::from_two_pos(
                        start + rect.min.to_vec2(),
                        end + rect.min.to_vec2(),
                    );
                    ui.painter().rect_stroke(
                        draw_rect,
                        0.0,
                        (2.0, egui::Color32::RED),
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }
    }
}
