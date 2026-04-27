use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Run OCR on a selected area and copy to clipboard immediately (Headless)
    #[arg(short, long)]
    pub extract: bool,

    /// Run OCR with semantic UI detection and copy to clipboard (Headless)
    #[arg(short, long)]
    pub semantic: bool,

    /// Specify the language for OCR (e.g., "eng", "spa")
    #[arg(short, long, default_value = "eng")]
    pub lang: String,

    /// Save the screenshot to a specific path (Headless)
    #[arg(short, long)]
    pub save: Option<String>,

    /// Save the screenshot to a specific directory with an auto-generated name (Headless)
    #[arg(short, long)]
    pub dir: Option<String>,

    /// Base directory for automatic saves (Default: ~/bloatshots)
    #[arg(long)]
    pub defaultfolder: Option<String>,

    /// Scale factor for image preprocessing (default: 2.0)
    #[arg(short, long, default_value = "2.0")]
    pub scale: f32,

    /// Open in editor immediately (Headless)
    #[arg(short, long)]
    pub edit: bool,
}
