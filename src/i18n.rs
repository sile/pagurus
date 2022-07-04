use std::time::Duration;

// BCP 47 (RFC 5646, RFC 4647)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct LanguageTag(String);

impl LanguageTag {
    pub fn new(lang: String) -> Self {
        Self(lang)
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct TimeZone {
    pub utc_offset: Duration,
}
