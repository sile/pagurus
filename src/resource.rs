use std::borrow::Cow;

// TODO: validation (e.g., must be a relative path, doesn't contain two many "..", etc)
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String")]
pub enum ResourceName {
    State(Cow<'static, str>),
    Asset(Cow<'static, str>),
    File(Cow<'static, str>), // TODO: Data or Blob
}

impl ResourceName {
    pub const fn state(name: &'static str) -> Self {
        Self::State(Cow::Borrowed(name))
    }

    pub const fn state_owned(name: String) -> Self {
        Self::State(Cow::Owned(name))
    }

    pub const fn asset(name: &'static str) -> Self {
        Self::Asset(Cow::Borrowed(name))
    }

    pub const fn asset_owned(name: String) -> Self {
        Self::Asset(Cow::Owned(name))
    }

    pub const fn file(file: &'static str) -> Self {
        Self::File(Cow::Borrowed(file))
    }

    pub const fn file_owned(file: String) -> Self {
        Self::File(Cow::Owned(file))
    }
}

impl std::fmt::Display for ResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::State(s) => write!(f, "state:{}", s),
            Self::Asset(s) => write!(f, "asset:{}", s),
            Self::File(s) => write!(f, "file:{}", s),
        }
    }
}

impl std::str::FromStr for ResourceName {
    type Err = ResourceNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("state:") {
            Ok(Self::state_owned(s["state:".len()..].to_owned()))
        } else if s.starts_with("asset:") {
            Ok(Self::asset_owned(s["asset:".len()..].to_owned()))
        } else if s.starts_with("file:") {
            Ok(Self::file_owned(s["file:".len()..].to_owned()))
        } else {
            Err(ResourceNameError::MalformedName { name: s.to_owned() })
        }
    }
}

impl TryFrom<String> for ResourceName {
    type Error = ResourceNameError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

// TODO(?): Use `Failure`?
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ResourceNameError {
    #[error("expected a resource name prefixed with \"(state|asset|file):\", but got {name:?}")]
    MalformedName { name: String },
}

// TODO(?): ResourceValue{ Blob(_), CrdtMap(_) }
