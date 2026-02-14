use serde::{Serialize, Deserialize, Serializer};
use std::fmt;

/// A wrapper for sensitive data that masks its value in Debug output and can be customized for Serialization.
#[derive(Clone, Deserialize)]
pub struct Masked<T>(pub T);

impl<T: fmt::Display> fmt::Debug for Masked<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "********")
    }
}

impl<T: fmt::Display> fmt::Display for Masked<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "********")
    }
}

impl<T: Serialize> Serialize for Masked<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // In logs, we might want to mask, but in API responses we need the real value.
        // This wrapper is primarily for preventing accidental leakage in log macros like tracing::info!("{:?}", event).
        self.0.serialize(serializer)
    }
}

impl<T> Masked<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}
