//! The one error shape the UI sees (spec §4.7): a kind, a plain-English
//! message, and for validation the list of failing fields.

use panowww_core::game::{FieldError, GameError};
use panowww_core::ports::{CaptureError, InputError};
use panowww_store::StoreError;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AppErrorKind {
    /// A Game broke the rules of spec §5; `fields` says which.
    Validation,
    NotFound,
    Store,
    Input,
    Capture,
    Permission,
    /// A run is going on; the request was ignored.
    Busy,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppError {
    pub kind: AppErrorKind,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fields: Option<Vec<FieldError>>,
}

impl AppError {
    pub fn new(kind: AppErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            fields: None,
        }
    }

    pub fn internal(message: impl std::fmt::Display) -> Self {
        Self::new(AppErrorKind::Internal, message.to_string())
    }

    pub fn not_found(what: impl std::fmt::Display) -> Self {
        Self::new(AppErrorKind::NotFound, format!("{what} was not found."))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

impl From<GameError> for AppError {
    fn from(error: GameError) -> Self {
        match error {
            GameError::Invalid(fields) => Self {
                kind: AppErrorKind::Validation,
                message: "Some fields are invalid.".to_owned(),
                fields: Some(fields),
            },
            other => Self::new(AppErrorKind::Store, other.to_string()),
        }
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Game {
                source: GameError::Invalid(fields),
                ..
            } => Self {
                kind: AppErrorKind::Validation,
                message: "Some fields are invalid.".to_owned(),
                fields: Some(fields),
            },
            StoreError::UnknownGame(id) => Self::not_found(format!("Game {id}")),
            other => Self::new(AppErrorKind::Store, other.to_string()),
        }
    }
}

impl From<CaptureError> for AppError {
    fn from(error: CaptureError) -> Self {
        let kind = match error {
            CaptureError::PermissionRevoked => AppErrorKind::Permission,
            _ => AppErrorKind::Capture,
        };
        Self::new(kind, error.to_string())
    }
}

impl From<InputError> for AppError {
    fn from(error: InputError) -> Self {
        let kind = match error {
            InputError::PermissionDenied => AppErrorKind::Permission,
            _ => AppErrorKind::Input,
        };
        Self::new(kind, error.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        Self::internal(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::internal(error)
    }
}

/// What every command returns.
pub type CmdResult<T> = Result<T, AppError>;
