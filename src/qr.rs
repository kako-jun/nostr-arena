//! QR code generation utilities

use qrcode::{QrCode, render::svg};

/// QR code options
#[derive(Debug, Clone, Default)]
pub struct QrOptions {
    /// Size/scale factor (pixels per module)
    pub size: Option<u32>,
    /// Margin (quiet zone in modules)
    pub margin: Option<u32>,
    /// Foreground color (hex)
    pub fg_color: Option<String>,
    /// Background color (hex)
    pub bg_color: Option<String>,
}

/// Generate QR code as SVG string
pub fn generate_qr_svg(data: &str, options: &QrOptions) -> Result<String, String> {
    let code = QrCode::new(data.as_bytes()).map_err(|e| e.to_string())?;

    let size = options.size.unwrap_or(4);
    let margin = options.margin.unwrap_or(2);
    let fg = options.fg_color.as_deref().unwrap_or("#000000");
    let bg = options.bg_color.as_deref().unwrap_or("#ffffff");

    let svg = code
        .render::<svg::Color>()
        .min_dimensions(size * 10, size * 10)
        .quiet_zone(margin > 0)
        .dark_color(svg::Color(fg))
        .light_color(svg::Color(bg))
        .build();

    Ok(svg)
}

/// Generate QR code as data URL
pub fn generate_qr_data_url(data: &str, options: &QrOptions) -> Result<String, String> {
    let svg = generate_qr_svg(data, options)?;
    let encoded = base64_encode(&svg);
    Ok(format!("data:image/svg+xml;base64,{encoded}"))
}

fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_svg() {
        let svg = generate_qr_svg("https://example.com", &QrOptions::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<path")); // Real QR code has path elements
    }

    #[test]
    fn test_generate_qr_data_url() {
        let url = generate_qr_data_url("test", &QrOptions::default()).unwrap();
        assert!(url.starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn test_qr_with_options() {
        let options = QrOptions {
            size: Some(8),
            margin: Some(4),
            fg_color: Some("#333333".to_string()),
            bg_color: Some("#ffffff".to_string()),
        };
        let svg = generate_qr_svg("https://example.com/room/abc123", &options).unwrap();
        assert!(svg.contains("#333333"));
    }
}
