# Bloatshot

A high-performance, hybrid CLI/GUI OCR screenshot utility specifically designed for Arch Linux and Hyprland.

Bloatshot bridges the gap between minimalist shell scripts and heavy GUI applications. Built in Rust, it provides a fast, interactive menu for common screenshot tasks while remaining fully scriptable via command-line flags.

## Key Features

- **Hybrid Interface**: Use the interactive GUI menu for manual tasks or headless flags for keyboard shortcuts.
- **Integrated OCR**: Extracts text from any screen region using the Tesseract engine.
- **Image Pre-processing**: Automatically applies grayscale conversion, upscaling, and binarization to maximize OCR accuracy.
- **Organized Storage**: Automatically saves screenshots into timestamped directories (`~/bloatshots/YYYY-MM-DD/`).
- **Rich Notifications**: Sends system notifications with image previews and file paths.
- **Wayland Native**: Built specifically for Wayland/Hyprland using `grim` and `slurp`.
- **Minimalist Aesthetic**: Features a compact, rounded UI that integrates seamlessly with modern tiling window managers.

## Prerequisites

Ensure the following dependencies are installed on your system:

- `grim` (Screenshot capture)
- `slurp` (Region selection)
- `tesseract` & `tesseract-data-eng` (OCR engine)
- `wl-clipboard` (Wayland clipboard support)
- `libnotify` (For notifications)

## Installation

### From Source (Recommended for Developers)

```bash
cargo install --path .
```

*Note: Ensure `~/.cargo/bin` is in your PATH.*

### Using PKGBUILD (Standard Arch Way)

```bash
makepkg -si
```

This installs the binary to `/usr/bin/bloatshot`, making it available system-wide.

## Hyprland Configuration (2026 Syntax)

To ensure the interactive menu appears as a floating utility window, add these rules to your `hyprland.conf`:

```ini
# Bloatshot Window Rules
windowrule = match:class ^(Bloatshot)$, float on
windowrule = match:class ^(Bloatshot)$, center on
windowrule = match:class ^(Bloatshot)$, stay_focused on
windowrule = match:class ^(Bloatshot)$, pin on

# Keybinds
bind = $mainMod, B, exec, bloatshot
bind = $mainMod SHIFT, S, exec, bloatshot --dir ~/bloatshots
bind = $mainMod ALT, S, exec, bloatshot -l eng+por --extract
```

## Usage

### Interactive Mode

Running `bloatshot` without action flags opens the utility menu:

1. **Extract Text**: Select an area to run OCR and copy text to the clipboard.
2. **Save Image**: Select an area to save a timestamped screenshot to `~/bloatshots`.
3. **Edit Image**: Select an area and immediately open it in your default image editor.
4. **See Image**: Select an area to preview the capture and perform sub-region OCR.

### Command Line Flags

- `--extract`: Capture a region and copy OCR text immediately.
- `--edit`: Capture a region and open in the default editor.
- `--save <path>`: Save capture to a specific file.
- `--dir <path>`: Save capture to a specific directory using auto-naming.
- `--defaultfolder <path>`: Override the default `~/bloatshots` base directory.
- `--lang <lang>`: Set the OCR language (default: `eng`).
- `--scale <factor>`: Set the pre-processing upscale factor (default: `2.0`).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
