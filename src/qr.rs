use anyhow::{Context, Result};
use qrcodegen::{QrCode, QrCodeEcc};
use std::path::Path;

/// Render a text string (such as an OTPAuth URL) as a terminal-scannable Unicode block QR code.
///
/// Uses standard Unicode half-block characters (▀, ▄, █, and space) to render 2 vertical modules per character cell,
/// matching standard terminal aspect ratio. The border provides the required quiet zone.
///
/// By default, dark terminal mode (`invert = true`) prints white/light modules on dark terminal background.
/// If `invert = false`, it prints dark modules on a light background.
pub fn render_terminal_qr(text: &str, invert: bool) -> Result<String> {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium)
        .map_err(|e| anyhow::anyhow!("Failed to encode QR code: {e:?}"))?;

    let size = qr.size();
    let border = 2; // quiet zone in modules
    let mut out = String::new();

    // Iterate 2 vertical modules at a time
    let mut y = -border;
    while y < size + border {
        for x in -border..size + border {
            let top_dark = if (0..size).contains(&x) && (0..size).contains(&y) {
                qr.get_module(x, y)
            } else {
                false
            };

            let bottom_y = y + 1;
            let bottom_dark = if (0..size).contains(&x) && (0..size).contains(&bottom_y) {
                qr.get_module(x, bottom_y)
            } else {
                false
            };

            // In terminal dark mode (invert = true):
            // Default terminal cell is dark (empty space).
            // A dark module in QR code needs to be "dark", but on a black screen,
            // standard scanners expect dark modules against a light background.
            // Using qrencode UTF8i convention:
            // Light (quiet zone / background) = full block / foreground.
            // Dark (QR module) = space (terminal background).
            //
            // Let's test standard vs inverted convention:
            // If invert == true (qrencode UTF8i for dark background terminals):
            // - top light, bottom light => '█' (Full block)
            // - top light, bottom dark  => '▀' (Top half)
            // - top dark, bottom light  => '▄' (Bottom half)
            // - top dark, bottom dark   => ' ' (Space)
            //
            // If invert == false (standard UTF8 for light background terminals):
            // - top dark, bottom dark   => '█' (Full block)
            // - top dark, bottom light  => '▀' (Top half)
            // - top light, bottom dark  => '▄' (Bottom half)
            // - top light, bottom light => ' ' (Space)
            let ch = if invert {
                match (!top_dark, !bottom_dark) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                }
            } else {
                match (top_dark, bottom_dark) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                }
            };
            out.push(ch);
        }
        out.push('\n');
        y += 2;
    }

    Ok(out)
}

/// Render one or more migration QR codes in the terminal with batch progress headers
pub fn render_migration_terminal_qrs(uris: &[String], invert: bool) -> Result<String> {
    if uris.is_empty() {
        return Ok(String::new());
    }
    let mut out = String::new();
    let total = uris.len();
    for (i, uri) in uris.iter().enumerate() {
        if total > 1 {
            out.push_str(&format!("[Migration QR {} of {}]\n", i + 1, total));
        }
        let qr = render_terminal_qr(uri, invert)?;
        out.push_str(&qr);
        if total > 1 && i + 1 < total {
            out.push('\n');
        }
    }
    Ok(out)
}

/// Scan and decode the first QR code found in an image file (PNG, JPEG, etc.).
pub fn scan_qr_from_image(path: &Path) -> Result<String> {
    let img = image::open(path)
        .with_context(|| format!("Failed to open QR image at '{}'", path.display()))?
        .into_luma8();

    let mut prepared = rqrr::PreparedImage::prepare(img);
    let grids = prepared.detect_grids();

    if grids.is_empty() {
        anyhow::bail!("No QR code detected in image '{}'", path.display());
    }

    let (_meta, content) = grids[0]
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode QR code in '{}': {e:?}", path.display()))?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_terminal_qr_output() {
        let text = "otpauth://totp/tfa:alice?secret=JBSWY3DPEHPK3PXP&issuer=tfa";
        let rendered_inverted = render_terminal_qr(text, true).unwrap();
        assert!(!rendered_inverted.is_empty());
        assert!(rendered_inverted.contains('█'));

        let rendered_std = render_terminal_qr(text, false).unwrap();
        assert!(!rendered_std.is_empty());
        assert!(rendered_std.contains('█'));
    }

    #[test]
    fn test_render_migration_terminal_qrs_single_and_multi() {
        let uri1 = "otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu8";
        let uri2 = "otpauth-migration://offline?data=CjMKCkhlbGxvId6tvu9";

        // Single URI: no multi-batch header
        let single = render_migration_terminal_qrs(&[uri1.to_string()], true).unwrap();
        assert!(!single.contains("[Migration QR"));
        assert!(single.contains('█'));

        // Multi URIs: includes batch headers
        let multi =
            render_migration_terminal_qrs(&[uri1.to_string(), uri2.to_string()], true).unwrap();
        assert!(multi.contains("[Migration QR 1 of 2]"));
        assert!(multi.contains("[Migration QR 2 of 2]"));
    }

    #[test]
    fn test_scan_qr_from_image_roundtrip() {
        let expected_payload =
            "otpauth://totp/tfa:bob?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=tfa";

        // Generate image using qrcodegen-image or image directly
        let qr = QrCode::encode_text(expected_payload, QrCodeEcc::Medium).unwrap();
        let size = qr.size() as u32;
        let border = 4u32;
        let img_dim = size + border * 2;
        let mut img_buf = image::GrayImage::new(img_dim, img_dim);

        // Fill background with white (255)
        for pixel in img_buf.pixels_mut() {
            *pixel = image::Luma([255u8]);
        }

        // Draw dark modules (0)
        for y in 0..size {
            for x in 0..size {
                if qr.get_module(x as i32, y as i32) {
                    img_buf.put_pixel(x + border, y + border, image::Luma([0u8]));
                }
            }
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let img_path = temp_dir.path().join("test_qr.png");
        img_buf.save(&img_path).unwrap();

        let decoded = scan_qr_from_image(&img_path).unwrap();
        assert_eq!(decoded, expected_payload);
    }

    #[test]
    fn test_scan_qr_nonexistent_image_fails() {
        let result = scan_qr_from_image(Path::new("/nonexistent/path/qr.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_qr_no_code_in_blank_image() {
        let img_buf = image::GrayImage::new(100, 100);
        let temp_dir = tempfile::tempdir().unwrap();
        let img_path = temp_dir.path().join("blank.png");
        img_buf.save(&img_path).unwrap();

        let result = scan_qr_from_image(&img_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No QR code detected"));
    }
}
