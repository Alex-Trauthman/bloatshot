use anyhow::{anyhow, Result};
use bloatshot::args::Args;
use bloatshot::gui::{AppMode, BloatshotApp, PendingAction};
use bloatshot::ocr::{perform_ocr, perform_ocr_with_semantic};
use bloatshot::screenshot::capture_screenshot;
use bloatshot::util::{get_auto_save_path, open_in_editor, resolve_path, send_notification};
use clap::Parser;
use eframe::egui;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn main() -> Result<()> {
    // Single instance lock
    let lock_path = std::env::temp_dir().join("bloatshot.lock");
    let _lock_file = File::create(&lock_path)?;
    if fs2::FileExt::try_lock_exclusive(&_lock_file).is_err() {
        println!("Bloatshot is already running.");
        return Ok(());
    }

    let args = Args::parse();

    if args.extract || args.semantic {
        return run_headless_extract(&args.lang, args.scale, args.semantic);
    }

    if args.edit {
        return run_headless_edit();
    }

    if let Some(path) = args.save {
        let p = resolve_path(&path);
        run_save(&p)?;
        send_notification("Screenshot Saved", &format!("{}", p.display()), Some(&p));
        return Ok(());
    }

    if let Some(dir) = args.dir {
        let path = get_auto_save_path(Some(&dir))?;
        run_save(&path)?;
        send_notification(
            "Screenshot Saved",
            &format!("{}", path.display()),
            Some(&path),
        );
        return Ok(());
    }

    let mut current_mode = AppMode::Menu;
    let mut current_image: Option<PathBuf> = None;

    loop {
        let pending_action = Arc::new(Mutex::new(None));
        run_gui(
            args.lang.clone(),
            args.scale,
            Arc::clone(&pending_action),
            current_mode,
            current_image.clone(),
        )?;

        let action = pending_action.lock().unwrap().take();
        if let Some(action) = action {
            std::thread::sleep(std::time::Duration::from_millis(250));
            match action {
                PendingAction::ExtractFull => {
                    run_headless_extract(&args.lang, args.scale, false)?;
                    break;
                }
                PendingAction::SemanticFull => {
                    run_headless_extract(&args.lang, args.scale, true)?;
                    break;
                }
                PendingAction::EditFull => {
                    run_headless_edit()?;
                    break;
                }
                PendingAction::SaveFull => {
                    let path = get_auto_save_path(args.defaultfolder.as_deref())?;
                    run_save(&path)?;
                    send_notification(
                        "Screenshot Saved",
                        &format!("{}", path.display()),
                        Some(&path),
                    );
                    break;
                }
                PendingAction::SeeImage => {
                    let path = std::env::temp_dir().join("bloatshot_capture.png");
                    if capture_screenshot(&path).is_ok() {
                        current_image = Some(path);
                        current_mode = AppMode::Viewer;
                    } else {
                        break;
                    }
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}

fn run_headless_extract(lang: &str, scale: f32, use_semantic: bool) -> Result<()> {
    let img_path = std::env::temp_dir().join("bloatshot_headless.png");
    capture_screenshot(&img_path)?;
    
    let text = if use_semantic {
        perform_ocr_with_semantic(&img_path, lang, scale)?
    } else {
        perform_ocr(&img_path, lang, scale)?
    };

    if !text.is_empty() {
        bloatshot::util::copy_to_clipboard(&text)?;
        send_notification("OCR Complete", "Text copied to clipboard", Some(&img_path));
    }
    Ok(())
}

fn run_headless_edit() -> Result<()> {
    let img_path = std::env::temp_dir().join("bloatshot_edit.png");
    capture_screenshot(&img_path)?;
    open_in_editor(&img_path)?;
    Ok(())
}

fn run_save(path: &Path) -> Result<()> {
    capture_screenshot(path)?;
    bloatshot::util::copy_image_to_clipboard(path)?;
    println!("{}", path.display());
    Ok(())
}

fn run_gui(
    lang: String,
    scale: f32,
    pending_action: Arc<Mutex<Option<PendingAction>>>,
    initial_mode: AppMode,
    initial_image: Option<PathBuf>,
) -> Result<()> {
    let (width, height) = if initial_mode == AppMode::Menu {
        (180.0, 180.0)
    } else {
        (600.0, 400.0)
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_always_on_top()
            .with_resizable(initial_mode == AppMode::Viewer)
            .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        "Bloatshot",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.global_style()).clone();

            style.visuals.window_corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

            style.spacing.button_padding = egui::vec2(10.0, 4.0);
            style.spacing.item_spacing = egui::vec2(0.0, 6.0);

            cc.egui_ctx.set_global_style(style);

            Ok(Box::new(BloatshotApp::new(
                lang,
                scale,
                pending_action,
                initial_mode,
                initial_image,
            )))
        }),
    )
    .map_err(|e| anyhow!("GUI error: {}", e))
}
