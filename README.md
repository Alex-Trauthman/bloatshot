# Bloatshot

A high-performance, professional-grade OCR screenshot utility specifically optimized for Arch Linux and Wayland/Hyprland.

Bloatshot bridges the gap between minimalist shell scripts and heavy GUI applications. Built in Rust and powered by state-of-the-art transformer models, it provides "Paper Perfect" extraction of standard text, mathematical formulas, and complex tables.

## Key Features

- **Multi-Modal OCR Excellence**:
    - **Standard Text**: Powered by PaddleOCR v5 for robust layout preservation.
    - **Math/LaTeX**: Powered by RapidLaTeXOCR (Transformer-based) with automatic `\left/\right` scaling for "Paper Perfect" professional typesetting.
    - **Table Intelligence**: Automatically detects and formats tables into balanced Markdown structures.
- **Hardware Accelerated**: Leverages MNN and ONNX Runtime for blazing-fast inference directly on your CPU/GPU.
- **Interactive & Headless**: Use the stunning interactive GUI for manual tasks or optimized flags for lightning-fast keyboard shortcuts.
- **Wayland Native**: Optimized for `grim` and `slurp` with clipboard integration via `wl-copy`.
- **Hybrid Viewer**: Inspect the last capture and perform targeted sub-region OCR instantly.

## Base Tools & Dependencies

Bloatshot relies on these core tools to function:

- **Screenshot & Region Selection**: `grim`, `slurp`
- **Image Processing**: `ImageMagick` (via Rust bindings), `imageproc`
- **Clipboard**: `wl-clipboard`
- **Notifications**: `libnotify`
- **Inference Engines**: `onnxruntime`, `MNN`
- **OCR Libraries**: `ocr-rs` (Standard), `ort` (Math/Table)

## Installation

### Using PKGBUILD (Recommended)

```bash
# Clone the repository and build the Arch package
makepkg -si
```

### From Source

```bash
cargo install --path .
```

## Usage

### Headless Shortcuts (Recommended for Keybinds)

- `bloatshot --extract` (`-e`): Capture region and copy **Standard Text** to clipboard.
- `bloatshot --semantic` (`-m`): Capture region and copy **Math/LaTeX** (Paper Perfect) to clipboard.
- `bloatshot --table` (`-t`): Capture region and copy **Markdown Table** to clipboard.
- `bloatshot --edit` (`-E`): Capture region and open immediately in your default editor.

### Interactive Menu

Simply run `bloatshot` to open the GUI.
- **Escape** or the **Close** button exits the app.
- **Save** button copies the capture to the clipboard and saves it to your directory.

## Hyprland Configuration

To ensure the utility menu behaves correctly as a floating tool, add these rules to your `hyprland.conf`:

```ini
# Bloatshot Floating Rules
windowrule = match:class ^(Bloatshot)$, float on
windowrule = match:class ^(Bloatshot)$, center on
windowrule = match:class ^(Bloatshot)$, stay_focused on
windowrule = match:class ^(Bloatshot)$, pin on

# Recommended Keybind Example
bind = $mainMod, B, exec, bloatshot
bind = $mainMod SHIFT, S, exec, bloatshot --dir ~/bloatshots
```

---

**Version 0.1.0**  
*Built for the Arch Linux community.*
