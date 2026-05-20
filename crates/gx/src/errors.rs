use thiserror::Error;

/// Typed error model. Maps the TS `CommandError(message, exitCode=1)` —
/// `Self::Command { exit_code }` covers any user-facing message; specific
/// variants will be added as concrete failure modes get ported.
#[derive(Debug, Error)]
pub enum GxError {
    #[error("{message}")]
    Command { message: String, exit_code: i32 },

    #[error("{0}")]
    Other(String),
}

impl GxError {
    pub fn command(message: impl Into<String>) -> Self {
        GxError::Command {
            message: message.into(),
            exit_code: 1,
        }
    }

    pub fn command_with_code(message: impl Into<String>, exit_code: i32) -> Self {
        GxError::Command {
            message: message.into(),
            exit_code,
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            GxError::Command { exit_code, .. } => *exit_code,
            GxError::Other(_) => 1,
        }
    }
}

pub type GxResult<T> = Result<T, GxError>;
