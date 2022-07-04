use std::{error::Error, panic::Location};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Failure {
    pub reason: String,
    pub backtrace: Vec<BacktraceItem>,
}

impl Failure {
    #[track_caller]
    pub fn new(reason: String) -> Self {
        let location = Location::caller();
        Self::with_location(reason, location)
    }

    pub fn with_location(reason: String, location: &Location) -> Self {
        Self {
            reason,
            backtrace: vec![BacktraceItem::new(location)],
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Failure: {}", self.reason)?;
        for item in &self.backtrace {
            writeln!(f, "  at {}:{}", item.file, item.line)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BacktraceItem {
    pub file: String,
    pub line: u32,
}

impl BacktraceItem {
    pub fn new(location: &Location) -> Self {
        Self {
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

pub trait OrFail {
    type Item;

    fn or_fail(self) -> Result<Self::Item, Failure>;
}

impl OrFail for bool {
    type Item = ();

    #[track_caller]
    fn or_fail(self) -> Result<Self::Item, Failure> {
        if self {
            Ok(())
        } else {
            let location = Location::caller();
            Err(Failure::with_location(
                format!("assertion failed"),
                location,
            ))
        }
    }
}

impl<T> OrFail for Option<T> {
    type Item = T;

    #[track_caller]
    fn or_fail(self) -> Result<Self::Item, Failure> {
        if let Some(item) = self {
            Ok(item)
        } else {
            let location = Location::caller();
            Err(Failure::with_location(
                format!("expected `Some(_)`, but got `None`"),
                location,
            ))
        }
    }
}

impl<T, E: Error> OrFail for Result<T, E> {
    type Item = T;

    #[track_caller]
    fn or_fail(self) -> Result<Self::Item, Failure> {
        match self {
            Ok(item) => Ok(item),
            Err(error) => {
                let location = Location::caller();
                Err(Failure::with_location(error.to_string(), location))
            }
        }
    }
}

impl<T> OrFail for Result<T, Failure> {
    type Item = T;

    #[track_caller]
    fn or_fail(self) -> Result<Self::Item, Failure> {
        match self {
            Ok(item) => Ok(item),
            Err(mut failure) => {
                let location = Location::caller();
                failure.backtrace.push(BacktraceItem::new(location));
                Err(failure)
            }
        }
    }
}
