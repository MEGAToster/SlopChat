use serde::Serialize;

// WebKitGTK does not bridge the system clipboard into the DOM clipboardData on
// Linux (and the Wayland layer surface makes it worse), so the composer reads
// the clipboard here instead. Other platforms use the working DOM path.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub image: Option<ClipboardImage>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardImage {
    pub png_base64: String,
}

#[cfg(target_os = "linux")]
pub fn read() -> Result<ClipboardContent, String> {
    use base64::Engine;

    let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
    if let Ok(text) = clipboard.get_text() {
        if !text.is_empty() {
            return Ok(ClipboardContent {
                text: Some(text),
                image: None,
            });
        }
    }

    let image = match clipboard.get_image() {
        Ok(image) => Some(ClipboardImage {
            png_base64: encode_png(&image)?,
        }),
        Err(_) => None,
    };
    Ok(ClipboardContent { text: None, image })
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
pub fn read() -> Result<ClipboardContent, String> {
    Ok(ClipboardContent {
        text: None,
        image: None,
    })
}
