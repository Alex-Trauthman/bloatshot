use crate::ocr::{perform_semantic_ocr, perform_standard_ocr, perform_table_ocr};
use crate::util::{copy_to_clipboard, open_in_editor, send_notification};
use anyhow::{Result, anyhow};
use eframe::egui;
use image::GenericImageView;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
pub enum AppMode {
    Menu,
    Viewer,
}

#[derive(PartialEq, Clone, Copy)]
pub enum OcrMode {
    Standard,
    Semantic,
    Table,
}

pub struct BloatshotApp {
    image_path: Option<PathBuf>,
    image: Option<image::DynamicImage>,
    preview_texture: Option<egui::TextureHandle>,
    ocr_result: Option<String>,
    selection_start: Option<egui::Pos2>,
    selection_end: Option<egui::Pos2>,
    pub is_selecting: bool,
    pub ocr_mode: OcrMode,
    pub mode: AppMode,
    pub pending_action: Arc<Mutex<Option<PendingAction>>>,
}

#[derive(Clone, Copy)]
pub enum PendingAction {
    ExtractFull,
    SemanticFull,
    TableFull,
    EditFull,
    SeeImage,
    SaveFull,
}

impl BloatshotApp {
    pub fn new(
        pending_action: Arc<Mutex<Option<PendingAction>>>,
        initial_mode: AppMode,
        initial_image: Option<PathBuf>,
    ) -> Self {
        let image = initial_image.as_ref().and_then(|p| image::open(p).ok());
        Self {
            image_path: initial_image,
            image,
            ocr_result: None,
            preview_texture: None,
            selection_start: None,
            selection_end: None,
            is_selecting: false,
            ocr_mode: OcrMode::Standard,
            mode: initial_mode,
            pending_action,
        }
    }

    fn run_ocr_on_selection(&mut self, rect: egui::Rect, image_size: egui::Vec2) -> Result<()> {
        let img = self.image.as_ref().ok_or(anyhow!("No image loaded"))?;
        let (img_w, img_h) = img.dimensions();

        let x = (rect.min.x / image_size.x * img_w as f32) as u32;
        let y = (rect.min.y / image_size.y * img_h as f32) as u32;
        let w = (rect.width() / image_size.x * img_w as f32) as u32;
        let h = (rect.height() / image_size.y * img_h as f32) as u32;

        if w == 0 || h == 0 {
            return Ok(());
        }

        let sub_img = img.view(x, y, w, h).to_image();
        let dynamic_sub = image::DynamicImage::ImageRgba8(sub_img);

        let text = match self.ocr_mode {
            OcrMode::Semantic => perform_semantic_ocr(&dynamic_sub)?,
            OcrMode::Table => perform_table_ocr(&dynamic_sub)?,
            OcrMode::Standard => perform_standard_ocr(&dynamic_sub)?,
        };

        copy_to_clipboard(&text)?;
        self.ocr_result = Some(text);

        send_notification("OCR Complete", "Text copied to clipboard", None);
        Ok(())
    }

    fn draw_menu(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("🚀 Bloatshot");
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.add_space(6.0);
                if ui.button("📋 Standard Text Extract").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::ExtractFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("📐 Math/LaTeX Extract").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::SemanticFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("📊 Table Extract").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::TableFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("🎨 Open Editor").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::EditFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("💾 Save Full").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::SaveFull);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }

                ui.add_space(6.0);
                if ui.button("👁 View Last").clicked() {
                    let mut action = self.pending_action.lock().unwrap();
                    *action = Some(PendingAction::SeeImage);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }

    fn draw_viewer(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let texture = self.preview_texture.get_or_insert_with(|| {
            let img = self.image.as_ref().unwrap();
            let size = [img.width() as usize, img.height() as usize];
            let pixels = img.to_rgba8();
            ctx.load_texture(
                "preview",
                egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_raw()),
                Default::default(),
            )
        });

        let img_size = texture.size_vec2();
        let texture_id = texture.id();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.ocr_mode, OcrMode::Standard, "Text");
                ui.selectable_value(&mut self.ocr_mode, OcrMode::Semantic, "Math");
                ui.selectable_value(&mut self.ocr_mode, OcrMode::Table, "Table");

                ui.separator();

                if ui.button("Edit Full").clicked()
                    && let Some(path) = &self.image_path
                {
                    open_in_editor(path).ok();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

            ui.separator();

            let available_size = ui.available_size();
            let ratio = (available_size.x / img_size.x).min(available_size.y / img_size.y);
            let draw_size = img_size * ratio;

            let (rect, response) = ui.allocate_exact_size(draw_size, egui::Sense::drag());

            ui.painter().image(
                texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            if response.drag_started() {
                self.selection_start = response.interact_pointer_pos();
                self.is_selecting = true;
            }

            if self.is_selecting {
                self.selection_end = response.interact_pointer_pos();

                if let (Some(start), Some(end)) = (self.selection_start, self.selection_end) {
                    let selection_rect = egui::Rect::from_two_pos(start, end);
                    let draw_rect = selection_rect.intersect(rect);

                    ui.painter().rect_stroke(
                        draw_rect,
                        0.0,
                        (2.0, egui::Color32::from_rgb(0, 255, 255)),
                        egui::StrokeKind::Outside,
                    );

                    if response.drag_stopped() {
                        self.is_selecting = false;
                        self.run_ocr_on_selection(
                            draw_rect.translate(-rect.min.to_vec2()),
                            draw_size,
                        )
                        .ok();
                    }
                }
            }

            if let Some(text) = &self.ocr_result {
                ui.add_space(10.0);
                ui.label("OCR Result (copied):");
                ui.code(text);
            }
        });
    }
}

impl eframe::App for BloatshotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.mode {
            AppMode::Menu => self.draw_menu(ui),
            AppMode::Viewer => self.draw_viewer(ui),
        }
    }
}
