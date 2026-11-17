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

/// `{{ value | title }}`
pub fn title<T: Display>(s: T) -> askama::Result<String> {
    let result = s
        .to_string()
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok(result)
}

/// `{{ value | trim_matches }}` - strips wrapping quotes left over from JSON
/// encoded settings.
pub fn trim_matches<T: Display>(s: T) -> askama::Result<String> {
    Ok(s.to_string().trim_matches('"').to_string())
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

/// Convenience fallback used by templates via `value.or(default)`.
pub trait Or<T> {
    type Output;
    fn or(self, default: T) -> Self::Output;
}

impl Or<String> for String {
    type Output = String;
    fn or(self, default: String) -> String {
        if self.is_empty() {
            default
        } else {
            self
        }
    }
}

impl Or<&str> for String {
    type Output = String;
    fn or(self, default: &str) -> String {
        if self.is_empty() {
            default.to_string()
        } else {
            self
        }
    }
}

impl Or<&str> for Option<String> {
    type Output = String;
    fn or(self, default: &str) -> String {
        self.filter(|value| !value.is_empty())
            .unwrap_or_else(|| default.to_string())
    }
}

impl Or<String> for Option<String> {
    type Output = String;
    fn or(self, default: String) -> String {
        self.filter(|value| !value.is_empty()).unwrap_or(default)
    }
}

impl Or<i32> for i32 {
    type Output = i32;
    fn or(self, default: i32) -> i32 {
        if self == 0 {
            default
        } else {
            self
        }
    }
}
