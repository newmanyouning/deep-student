/// 检测Base64编码图像的真实格式
pub(crate) fn detect_image_format_from_base64(base64_data: &str) -> &'static str {
    use base64::{engine::general_purpose, Engine as _};
    if let Ok(decoded) =
        general_purpose::STANDARD.decode(base64_data.get(..100).unwrap_or(base64_data))
    {
        detect_image_format_from_bytes(&decoded)
    } else {
        "jpeg"
    }
}

/// 根据图像字节数据检测格式
pub(crate) fn detect_image_format_from_bytes(image_data: &[u8]) -> &'static str {
    if image_data.len() < 4 {
        return "jpeg";
    }

    if image_data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpeg"
    } else if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        "png"
    } else if image_data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
        "gif"
    } else if image_data.len() >= 12
        && image_data.starts_with(&[0x52, 0x49, 0x46, 0x46])
        && &image_data[8..12] == &[0x57, 0x45, 0x42, 0x50]
    {
        "webp"
    } else if image_data.starts_with(&[0x42, 0x4D]) {
        "bmp"
    } else {
        "jpeg"
    }
}
