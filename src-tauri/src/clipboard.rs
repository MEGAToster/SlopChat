use serde::Serialize;

use crate::store::attachments::SavedFile;

// WebKitGTK does not bridge the system clipboard into the DOM clipboardData on
// Linux (and the Wayland layer surface makes it worse), so the composer reads
// the clipboard here instead. Other platforms use the working DOM path.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub image: Option<ClipboardImage>,
    // Set instead of `text` when the paste is too large to send inline: it has
    // already been written to the attachments directory.
    pub file: Option<SavedFile>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardImage {
    pub png_base64: String,
}

// Mirrors src/lib/paste.ts so a huge paste never reaches the webview.
#[cfg(any(target_os = "linux", test))]
const PASTE_AS_FILE_HARD_CAP: usize = 1_000_000;

#[cfg(any(target_os = "linux", test))]
fn should_store(len: usize, limit: usize) -> bool {
    len >= PASTE_AS_FILE_HARD_CAP || (limit > 0 && len >= limit)
}

#[cfg(target_os = "linux")]
pub fn read(limit: usize) -> Result<ClipboardContent, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
    if let Ok(text) = clipboard.get_text() {
        if !text.is_empty() {
            if should_store(text.len(), limit) {
                let file =
                    crate::store::attachments::save_bytes("paste.txt", "text/plain", text.as_bytes())?;
                return Ok(ClipboardContent {
                    text: None,
                    image: None,
                    file: Some(file),
                });
            }
            return Ok(ClipboardContent {
                text: Some(text),
                image: None,
                file: None,
            });
        }
    }

    let image = match clipboard.get_image() {
        Ok(image) => Some(ClipboardImage {
            png_base64: encode_png(&image)?,
        }),
        Err(_) => None,
    };
    Ok(ClipboardContent {
        text: None,
        image,
        file: None,
    })
}

#[cfg(target_os = "linux")]
fn encode_png(image: &arboard::ImageData) -> Result<String, String> {
    use base64::Engine;

    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, image.width as u32, image.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        writer
            .write_image_data(&image.bytes[..])
            .map_err(|error| error.to_string())?;
    }
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

#[cfg(not(target_os = "linux"))]
pub fn read(_limit: usize) -> Result<ClipboardContent, String> {
    Ok(ClipboardContent {
        text: None,
        image: None,
        file: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{should_store, PASTE_AS_FILE_HARD_CAP};

    #[test]
    fn small_text_stays_inline() {
        assert!(!should_store(10, 10_000));
    }

    #[test]
    fn text_at_the_limit_is_stored() {
        assert!(should_store(10_000, 10_000));
    }

    #[test]
    fn disabled_limit_still_caps_at_the_hard_limit() {
        assert!(!should_store(10_000, 0));
        assert!(should_store(PASTE_AS_FILE_HARD_CAP, 0));
    }
}
