use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TrunkError {
    pub code: String,
    pub message: String,
}

impl TrunkError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        TrunkError {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Serialize to the JSON string a Tauri command returns as its `Err` payload.
    /// Serializing a two-string struct cannot realistically fail; the fallback
    /// avoids a panic on the impossible case instead of `.unwrap()`.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            String::from(r#"{"code":"serialize_error","message":"failed to serialize error"}"#)
        })
    }
}

impl From<git2::Error> for TrunkError {
    fn from(e: git2::Error) -> Self {
        TrunkError {
            code: "git_error".into(),
            message: e.message().to_owned(),
        }
    }
}
