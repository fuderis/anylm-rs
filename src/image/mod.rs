use crate::{prelude::*, utils};

use base64::{Engine as _, engine};
use std::{fs, path::Path};

/// Image base64 url.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Image {
    pub url: String,
}

impl Image {
    /// Creates new base64 image from file path.
    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Image> {
        // reading file
        let file_path = file_path.as_ref();
        let file_content = fs::read(&file_path)?;

        // reading mime-type
        let mime_type = match file_path.extension().and_then(|e| e.to_str()) {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            _ => "application/octet-stream",
        };

        // encoding into base64
        let encoded = engine::general_purpose::STANDARD.encode(&file_content);
        let url = str!("data:{};base64,{}", mime_type, encoded);

        Ok(Image { url })
    }

    /// Creates new base64 image url (example: "data:image/png;base64,iVBORw0KGgoA...").
    pub fn from_base64(base64_url: impl Into<String>) -> Result<Image> {
        let url = base64_url.into();

        if utils::validate_base64(&url.split_once(",").ok_or(Error::InvalidBase64Url)?.1) {
            Ok(Image { url })
        } else {
            Err(Error::InvalidBase64Url.into())
        }
    }
}
