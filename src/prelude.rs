#![allow(unused_imports)]
pub(crate) use crate::error::Error;
pub(crate) use macron::*;

/// The dynamic error type
pub(crate) type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;
/// The short result alias
pub(crate) type Result<T> = std::result::Result<T, DynError>;
/// The std result alias
pub(crate) use std::result::Result as StdResult;

pub use bytes::Bytes;
pub use pearce::Proxy;

pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use serde_json::{self as json, Value as JsonValue, json};
pub(crate) use std::collections::{HashMap, HashSet};
