//! Custom Askama filters and helper traits.
//!
//! Askama 0.12 resolves unknown filters to `filters::<name>(...)` in the scope
//! of the template struct, so importing this module in the handler files makes
//! the custom filters available to templates.

use std::fmt::Display;

/// `{{ value | replace("a", "b") }}`
pub fn replace<T: Display, F: Display, R: Display>(s: T, from: F, to: R) -> askama::Result<String> {
    Ok(s.to_string().replace(&from.to_string(), &to.to_string()))
}

/// `{{ bytes | filesize }}`
pub fn filesize(size: &i64) -> askama::Result<String> {
    let size = (*size).max(0) as f64;
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = size;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        Ok(format!("{} {}", value as i64, UNITS[unit]))
    } else {
        Ok(format!("{:.1} {}", value, UNITS[unit]))
    }
}

/// `{% set items = value | from_json %}` - deserialize a JSON array/string.
pub fn from_json(s: &str) -> askama::Result<Vec<String>> {
    serde_json::from_str(s).map_err(|err| askama::Error::Custom(Box::new(err)))
}

