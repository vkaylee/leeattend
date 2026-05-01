use rust_xlsxwriter::{Color, Format, FormatAlign};

/// Common Excel Formatting Utilities
pub struct ExcelFormatUtils;

impl ExcelFormatUtils {
    pub fn title_format() -> Format {
        Format::new()
            .set_bold()
            .set_font_size(14)
            .set_font_color(Color::RGB(0x1F4E79))
    }

    pub fn subtitle_format() -> Format {
        Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x5B5B5B))
    }

    pub fn header_format() -> Format {
        Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x4472C4))
            .set_font_color(Color::RGB(0xFFFFFF))
    }

    pub fn number_format() -> Format {
        Format::new().set_align(FormatAlign::Right)
    }

    pub fn error_header_format() -> Format {
        Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0xFCE4D6))
            .set_font_color(Color::RGB(0xC00000))
    }

    pub fn error_row_format() -> Format {
        Format::new().set_background_color(Color::RGB(0xFDEDEC))
    }

    pub fn error_message_format() -> Format {
        Format::new()
            .set_font_color(Color::RGB(0xD32F2F))
            .set_bold()
    }

    pub fn example_row_format() -> Format {
        Format::new()
            .set_italic()
            .set_font_color(Color::RGB(0x5B5B5B))
    }

    pub fn ref_header_format() -> Format {
        Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(Color::RGB(0x2E7D32))
            .set_font_color(Color::RGB(0xFFFFFF))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_creations() {
        // Just verify that they create Formats without panic
        let _ = ExcelFormatUtils::title_format();
        let _ = ExcelFormatUtils::subtitle_format();
        let _ = ExcelFormatUtils::header_format();
        let _ = ExcelFormatUtils::number_format();
        let _ = ExcelFormatUtils::error_header_format();
        let _ = ExcelFormatUtils::error_row_format();
        let _ = ExcelFormatUtils::error_message_format();
        let _ = ExcelFormatUtils::example_row_format();
        let _ = ExcelFormatUtils::ref_header_format();
    }
}

