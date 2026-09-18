use base64::{Engine as _, engine};

/// Validates `base64` URL.
pub fn validate_base64(base64_url: &str) -> bool {
    engine::general_purpose::STANDARD.decode(base64_url).is_ok()
}
