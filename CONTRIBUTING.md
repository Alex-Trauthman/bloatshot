# Contributing to Bloatshot

Thank you for your interest in contributing to Bloatshot! This project aims to be a minimalist, high-performance OCR screenshot utility for Arch Linux and Hyprland.

## Development Setup

### Prerequisites

You will need the following installed:

- **Rust**: [Installation guide](https://www.rust-lang.org/tools/install)
- **Wayland Tools**: `sudo pacman -S grim slurp wl-clipboard`
- **Other**: `sudo pacman -S libnotify clang onnxruntime`

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/Alex-Trauthman/bloatshot.git
   cd bloatshot
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

Note: On the first run, the application will download necessary OCR and Math models (~200MB) to `~/.local/share/bloatshot`.

## Project Structure

- `src/main.rs`: Entry point and process lifecycle management.
- `src/gui.rs`: Interactive menu and viewer (egui).
- `src/ocr.rs`: High-performance OCR pipeline (PaddleOCR v5 + RapidLaTeXOCR).
- `src/screenshot.rs`: Wrapper for `grim` and `slurp`.
- `src/util.rs`: Model management, notification, and clipboard utilities.
- `src/args.rs`: CLI argument definitions.

## Testing

We use a Test-Driven Development (TDD) approach for the OCR pipeline. Before submitting a PR, ensure all tests pass:

```bash
cargo test -- --nocapture
```

The test suite validates:
- Standard multilingual text extraction.
- LaTeX formula reconstruction accuracy.
- Table layout analysis.

## Coding Standards

- **Formatting**: Always run `cargo fmt` before committing.
- **Error Handling**: Use `anyhow` for application-level errors and ensure informative error messages.
- **Documentation**: Use Rustdoc (`///`) for all public functions and modules.

## Submitting Changes

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes (`git commit -m 'Add amazing feature'`).
4. Push to the branch (`git push origin feature/amazing-feature`).
5. Open a Pull Request.

## License

By contributing, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).
