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

impl std::fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for LanguageTag {
    type Err = LanguageTagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LanguageTagError {}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct TimeZone {
    pub utc_offset: Duration,
}

impl TimeZone {
    pub const UTC: Self = TimeZone {
        utc_offset: Duration::from_secs(0),
    };
}

impl Default for TimeZone {
    fn default() -> Self {
        Self::UTC
    }
}
