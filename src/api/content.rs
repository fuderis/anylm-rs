use crate::{image, prelude::*};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// The image base64 url
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Image {
    pub url: String,
}

/// The message content
#[derive(From, Debug, Clone, Eq, PartialEq)]
#[from(Bytes, expr = Content::text(String::from_utf8_lossy(&value)))]
#[from(String, with = Content::text)]
#[from(&str, with = Content::text)]
#[from(Cow<'_, str>, with = Content::text)]
#[from(&Path, expr = Content::image_file(value, None).unwrap())]
#[from(PathBuf, expr = Content::image_file(value, None).unwrap())]
pub enum Content {
    #[from(skip)]
    Text {
        /// The message text
        text: String,
    },

    #[from(skip)]
    Image {
        /// The image base64 url
        image: Image,
        /// The image detail level (low/high/auto)
        detail: Option<String>,
    },
}

impl Content {
    /// Creates a new text content
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Creates a new image base64 url content
    pub fn image_url(base64: impl Into<String>, detail: Option<String>) -> Result<Self> {
        Ok(Self::Image {
            image: Image {
                url: image::base64(base64)?,
            },
            detail,
        })
    }

    /// Reads image file as base64 url
    pub fn image_file(path: impl AsRef<Path>, detail: Option<String>) -> Result<Self> {
        Ok(Self::Image {
            image: Image {
                url: image::read(path.as_ref())?,
            },
            detail,
        })
    }

    /// Returns true if text content is empty
    pub fn is_empty(&self) -> bool {
        if let Self::Text { text } = self {
            text.is_empty()
        } else {
            false
        }
    }
}

impl ::serde::Serialize for Content {
    fn serialize<S>(&self, se: S) -> StdResult<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        match self {
            Content::Text { text } => {
                let mut s = se.serialize_struct("Content", 2)?;
                s.serialize_field("type", "text")?;
                s.serialize_field("text", text)?;
                s.end()
            }
            Content::Image { image, detail } => {
                let mut s = se.serialize_struct("Content", 2)?;
                s.serialize_field("type", "image_url")?;
                s.serialize_field("image_url", image)?;
                if let Some(detail) = detail {
                    s.serialize_field("detail", detail)?;
                }
                s.end()
            }
        }
    }
}

struct ContentVisitor;

impl<'de> serde::de::Visitor<'de> for ContentVisitor {
    type Value = Content;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("struct Content with type field")
    }

    fn visit_map<V>(self, mut map: V) -> StdResult<Self::Value, V::Error>
    where
        V: ::serde::de::MapAccess<'de>,
    {
        let mut ctype: Option<String> = None;
        let mut text: Option<String> = None;
        let mut image_url: Option<Image> = None;
        let mut detail: Option<String> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "type" => {
                    if ctype.is_some() {
                        return Err(serde::de::Error::duplicate_field("type"));
                    }
                    ctype = Some(map.next_value()?);
                }
                "text" => {
                    if text.is_some() {
                        return Err(serde::de::Error::duplicate_field("text"));
                    }
                    text = Some(map.next_value()?);
                }
                "image_url" => {
                    if image_url.is_some() {
                        return Err(serde::de::Error::duplicate_field("image_url"));
                    }
                    image_url = Some(map.next_value()?);
                }
                "detail" => {
                    if detail.is_some() {
                        return Err(serde::de::Error::duplicate_field("detail"));
                    }
                    detail = Some(map.next_value()?);
                }
                _ => {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let ctype = ctype.ok_or_else(|| serde::de::Error::missing_field("type"))?;

        match ctype.as_str() {
            "text" => {
                let text = text.ok_or_else(|| serde::de::Error::missing_field("text"))?;
                Ok(Content::Text { text })
            }
            "image_url" => {
                let image_url =
                    image_url.ok_or_else(|| serde::de::Error::missing_field("image_url"))?;
                Ok(Content::Image {
                    image: image_url,
                    detail,
                })
            }
            _ => Err(serde::de::Error::unknown_variant(
                &ctype,
                &["text", "image_url"],
            )),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Content {
    fn deserialize<D>(de: D) -> StdResult<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["type", "text", "image_url", "detail"];
        de.deserialize_struct("Content", FIELDS, ContentVisitor)
    }
}
