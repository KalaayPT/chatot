pub mod charmap;
pub mod decode;
pub mod encode;
pub mod error;
pub mod format;

pub use charmap::{Charmap, get_default_charmap};
pub use decode::{TextArchive, decode_archive};
pub use encode::{DiagnosticContext, ErrorFormat, validate_message};
pub use error::FormatError;
pub use format::{
    DIALOG_LINE_MAX_PX, default_glyph_widths, line_is_too_long, measure_line_width, word_wrap,
};

// Define common types used across modules
use std::path::PathBuf;

#[derive(Clone)]
pub struct BinarySource {
    pub archive: Option<Vec<PathBuf>>,
    pub archive_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub struct TextSource {
    pub txt: Option<Vec<PathBuf>>,
    pub text_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Settings {
    pub json: bool,
    pub lang: String,
    pub newer_only: bool,
    pub msgenc_format: bool,
}
