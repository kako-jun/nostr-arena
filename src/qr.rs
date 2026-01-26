//! QR code generation utilities

/// QR code options
#[derive(Debug, Clone, Default)]
pub struct QrOptions {
    /// Size/scale factor
    pub size: Option<u32>,
    /// Margin (quiet zone)
    pub margin: Option<u32>,
    /// Foreground color (hex)
    pub fg_color: Option<String>,
    /// Background color (hex)
    pub bg_color: Option<String>,
}

/// Generate QR code as SVG string
pub fn generate_qr_svg(data: &str, options: &QrOptions) -> Result<String, String> {
    let size = options.size.unwrap_or(200);
    let margin = options.margin.unwrap_or(4);
    let fg = options.fg_color.as_deref().unwrap_or("#000000");
    let bg = options.bg_color.as_deref().unwrap_or("#ffffff");

    // Placeholder SVG - real implementation would use qrcode crate
    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">
  <rect width="100%" height="100%" fill="{bg}"/>
  <rect x="{margin}" y="{margin}" width="{w}" height="{w}" fill="{fg}" rx="4"/>
  <text x="50%" y="50%" fill="{bg}" text-anchor="middle" dominant-baseline="middle" font-size="10" font-family="monospace">{data}</text>
</svg>"#,
        size = size,
        margin = margin,
        w = size - margin * 2,
        fg = fg,
        bg = bg,
        data = if data.len() > 30 { &data[..30] } else { data }
    ))
}

/// Generate QR code as data URL
pub fn generate_qr_data_url(data: &str, options: &QrOptions) -> Result<String, String> {
    let svg = generate_qr_svg(data, options)?;
    let encoded = base64_encode(&svg);
    Ok(format!("data:image/svg+xml;base64,{}", encoded))
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
    }

    #[test]
    fn test_generate_qr_data_url() {
        let url = generate_qr_data_url("test", &QrOptions::default()).unwrap();
        assert!(url.starts_with("data:image/svg+xml;base64,"));
    }
}
